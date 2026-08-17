package dev.autodev.commandcenter

import kotlinx.coroutines.Dispatchers
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

private class ViewModelCodexOperations : CodexOperations {
    val calls = mutableListOf<String>()

    override fun account(endpoint: String): CodexAccountInfo {
        calls += "account:$endpoint"
        return CodexAccountInfo(true, "chatgpt", "plus")
    }

    override fun startBrowserLogin(endpoint: String): CodexLoginPrompt.Browser {
        calls += "browser:$endpoint"
        return CodexLoginPrompt.Browser("browser-1", "https://chatgpt.com/auth")
    }

    override fun startDeviceCodeLogin(endpoint: String): CodexLoginPrompt.DeviceCode {
        calls += "device:$endpoint"
        return CodexLoginPrompt.DeviceCode(
            "device-1",
            "https://auth.openai.com/codex/device",
            "ABCD-1234",
        )
    }

    override fun rateLimits(endpoint: String): CodexRateLimitInfo {
        calls += "limits:$endpoint"
        return CodexRateLimitInfo("codex", "plus", 42, 1_800_000_000L, 2, "7.50")
    }

    override fun logout(endpoint: String) {
        calls += "logout:$endpoint"
    }
}

class CommandCenterViewModelCodexTest {
    private val endpoint = "http://10.0.2.2:8080"

    @Test
    fun account_refresh_publishes_codex_account_state() {
        val operations = ViewModelCodexOperations()
        val viewModel = viewModel(operations)

        viewModel.refreshCodexAccount(endpoint)

        assertEquals("plus", viewModel.codexState.value.account?.planType)
        assertEquals(listOf("account:$endpoint"), operations.calls)
    }

    @Test
    fun browser_and_device_login_publish_safe_prompts() {
        val operations = ViewModelCodexOperations()
        val viewModel = viewModel(operations)

        viewModel.startCodexBrowserLogin(endpoint)
        assertEquals(
            "https://chatgpt.com/auth",
            (viewModel.codexState.value.loginPrompt as CodexLoginPrompt.Browser).authUrl,
        )

        viewModel.startCodexDeviceCodeLogin(endpoint)
        assertEquals(
            "ABCD-1234",
            (viewModel.codexState.value.loginPrompt as CodexLoginPrompt.DeviceCode).userCode,
        )
    }

    @Test
    fun rate_limits_and_logout_update_codex_state() {
        val operations = ViewModelCodexOperations()
        val viewModel = viewModel(operations)

        viewModel.refreshCodexAccount(endpoint)
        viewModel.refreshCodexRateLimits(endpoint)
        assertEquals(42, viewModel.codexState.value.rateLimits?.usedPercent)

        viewModel.logoutCodex(endpoint)

        assertNull(viewModel.codexState.value.account)
        assertNull(viewModel.codexState.value.rateLimits)
        assertNull(viewModel.codexState.value.loginPrompt)
    }

    private fun viewModel(operations: CodexOperations): CommandCenterViewModel =
        CommandCenterViewModel(
            codexController = CodexPanelController(operations),
            codexDispatcher = Dispatchers.Unconfined,
        )
}
