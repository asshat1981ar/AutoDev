package dev.autodev.commandcenter

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private class RecordingCodexUrlLauncher : CodexUrlLauncher {
    val urls = mutableListOf<String>()

    override fun open(url: String) {
        urls += url
    }
}

class CodexPanelUiTest {
    @Test
    fun authenticated_plus_account_and_usage_map_to_safe_display_model() {
        val state =
            CodexPanelState(
                account = CodexAccountInfo(true, "chatgpt", "plus"),
                rateLimits = CodexRateLimitInfo("codex", "plus", 42, 1_800_000_000L, 2, "7.50"),
            )

        val model = codexPanelUiModel(state)

        assertEquals("Authenticated", model.authStatus)
        assertEquals("plus", model.planType)
        assertEquals(42, model.usedPercent)
        assertEquals(1_800_000_000L, model.resetsAt)
        assertEquals(2, model.resetCreditsAvailable)
        assertEquals("7.50", model.creditBalance)
        assertNull(model.browserAuthUrl)
        assertNull(model.deviceVerificationUrl)
        assertNull(model.deviceUserCode)
        assertNull(model.errorCode)
    }

    @Test
    fun browser_prompt_maps_only_safe_auth_url() {
        val model =
            codexPanelUiModel(
                CodexPanelState(
                    loginPrompt =
                        CodexLoginPrompt.Browser(
                            "login-1",
                            "https://chatgpt.com/auth",
                        ),
                ),
            )

        assertEquals("https://chatgpt.com/auth", model.browserAuthUrl)
        assertNull(model.deviceVerificationUrl)
        assertNull(model.deviceUserCode)
    }

    @Test
    fun device_code_prompt_maps_verification_url_and_user_code() {
        val state =
            CodexPanelState(
                loginPrompt =
                    CodexLoginPrompt.DeviceCode(
                        "login-1",
                        "https://auth.openai.com/codex/device",
                        "ABCD-1234",
                    ),
            )

        val model = codexPanelUiModel(state)

        assertNull(model.browserAuthUrl)
        assertEquals("https://auth.openai.com/codex/device", model.deviceVerificationUrl)
        assertEquals("ABCD-1234", model.deviceUserCode)
    }

    @Test
    fun browser_action_opens_only_https_browser_prompts() {
        val launcher = RecordingCodexUrlLauncher()
        val action = CodexBrowserAction(launcher)

        assertTrue(
            action.open(
                CodexLoginPrompt.Browser(
                    "login-1",
                    "https://chatgpt.com/auth",
                ),
            ),
        )
        assertEquals(listOf("https://chatgpt.com/auth"), launcher.urls)

        assertFalse(action.open(CodexLoginPrompt.Browser("login-2", "javascript:alert(1)")))
        assertFalse(
            action.open(
                CodexLoginPrompt.DeviceCode(
                    "login-3",
                    "https://auth.openai.com/codex/device",
                    "ABCD-1234",
                ),
            ),
        )
        assertEquals(listOf("https://chatgpt.com/auth"), launcher.urls)
    }

    @Test
    fun browser_action_opens_only_https_urls_from_ui_model() {
        val launcher = RecordingCodexUrlLauncher()
        val action = CodexBrowserAction(launcher)

        assertTrue(action.openUrl("https://chatgpt.com/auth"))
        assertFalse(action.openUrl(null))
        assertFalse(action.openUrl("http://chatgpt.com/auth"))
        assertFalse(action.openUrl("javascript:alert(1)"))
        assertFalse(action.openUrl("https:///missing-host"))

        assertEquals(listOf("https://chatgpt.com/auth"), launcher.urls)
    }
}
