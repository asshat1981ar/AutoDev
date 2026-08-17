package dev.autodev.commandcenter

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertSame

private class FakeCodexOperations : CodexOperations {
    var accountResult: Result<CodexAccountInfo> =
        Result.success(CodexAccountInfo(true, "chatgpt", "plus"))
    var browserResult: Result<CodexLoginPrompt.Browser> =
        Result.success(CodexLoginPrompt.Browser("login-1", "https://chatgpt.com/auth"))
    var deviceResult: Result<CodexLoginPrompt.DeviceCode> =
        Result.success(
            CodexLoginPrompt.DeviceCode(
                "login-2",
                "https://auth.openai.com/codex/device",
                "ABCD-1234",
            ),
        )
    var limitsResult: Result<CodexRateLimitInfo> =
        Result.success(CodexRateLimitInfo("codex", "plus", 42, 1_800_000_000L, 2, "7.50"))
    var logoutResult: Result<Unit> = Result.success(Unit)
    val calls = mutableListOf<String>()

    override fun account(endpoint: String): CodexAccountInfo {
        calls += "account:$endpoint"
        return accountResult.getOrThrow()
    }

    override fun startBrowserLogin(endpoint: String): CodexLoginPrompt.Browser {
        calls += "browser:$endpoint"
        return browserResult.getOrThrow()
    }

    override fun startDeviceCodeLogin(endpoint: String): CodexLoginPrompt.DeviceCode {
        calls += "device:$endpoint"
        return deviceResult.getOrThrow()
    }

    override fun rateLimits(endpoint: String): CodexRateLimitInfo {
        calls += "limits:$endpoint"
        return limitsResult.getOrThrow()
    }

    override fun logout(endpoint: String) {
        calls += "logout:$endpoint"
        logoutResult.getOrThrow()
    }
}

class CodexPanelControllerTest {
    @Test
    fun account_login_usage_and_logout_use_the_current_server_endpoint() {
        val operations = FakeCodexOperations()
        val controller = CodexPanelController(operations)
        val endpoint = "http://10.0.2.2:8080"

        val accountState = controller.refreshAccount(endpoint, CodexPanelState())
        val browserState = controller.startBrowserLogin(endpoint, accountState)
        val deviceState = controller.startDeviceCodeLogin(endpoint, browserState)
        val limitsState = controller.refreshRateLimits(endpoint, deviceState)
        val logoutState = controller.logout(endpoint, limitsState)

        assertEquals(
            listOf(
                "account:$endpoint",
                "browser:$endpoint",
                "device:$endpoint",
                "limits:$endpoint",
                "logout:$endpoint",
            ),
            operations.calls,
        )
        assertNull(logoutState.account)
        assertNull(logoutState.rateLimits)
        assertNull(logoutState.loginPrompt)
        assertFalse(logoutState.busy)
    }

    @Test
    fun api_errors_preserve_last_good_data_and_publish_only_the_error_code() {
        val operations = FakeCodexOperations()
        val controller = CodexPanelController(operations)
        val account = CodexAccountInfo(true, "chatgpt", "plus")
        val limits = CodexRateLimitInfo("codex", "plus", 42, null, null, null)
        val existing = CodexPanelState(account = account, rateLimits = limits)
        operations.accountResult = Result.failure(CodexApiException(null, "network_error"))

        val next = controller.refreshAccount("http://server", existing)

        assertSame(account, next.account)
        assertSame(limits, next.rateLimits)
        assertEquals("network_error", next.errorCode)
        assertFalse(next.busy)
    }

    @Test
    fun unexpected_operation_failures_are_sanitized_to_network_error() {
        val operations = FakeCodexOperations()
        val controller = CodexPanelController(operations)
        operations.limitsResult = Result.failure(IllegalStateException("secret socket detail"))

        val next = controller.refreshRateLimits("http://server", CodexPanelState())

        assertEquals("network_error", next.errorCode)
        assertFalse(next.busy)
    }
}
