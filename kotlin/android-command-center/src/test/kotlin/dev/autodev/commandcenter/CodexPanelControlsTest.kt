package dev.autodev.commandcenter

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class CodexPanelControlsTest {
    @Test
    fun authenticated_idle_state_enables_account_actions_and_logout() {
        val model =
            codexPanelUiModel(
                CodexPanelState(
                    account = CodexAccountInfo(true, "chatgpt", "plus"),
                    loginPrompt = CodexLoginPrompt.Browser("login-1", "https://chatgpt.com/auth"),
                ),
            )

        val controls = codexPanelControls(model)

        assertTrue(controls.canRefreshAccount)
        assertTrue(controls.canStartBrowserLogin)
        assertTrue(controls.canStartDeviceCodeLogin)
        assertTrue(controls.canRefreshUsage)
        assertTrue(controls.canOpenBrowser)
        assertTrue(controls.canLogout)
    }

    @Test
    fun busy_state_disables_all_provider_actions() {
        val model = codexPanelUiModel(CodexPanelState(busy = true))

        val controls = codexPanelControls(model)

        assertFalse(controls.canRefreshAccount)
        assertFalse(controls.canStartBrowserLogin)
        assertFalse(controls.canStartDeviceCodeLogin)
        assertFalse(controls.canRefreshUsage)
        assertFalse(controls.canOpenBrowser)
        assertFalse(controls.canLogout)
    }

    @Test
    fun unauthenticated_state_keeps_login_available_but_disables_logout_and_open_browser() {
        val model =
            codexPanelUiModel(
                CodexPanelState(account = CodexAccountInfo(false, null, null)),
            )

        val controls = codexPanelControls(model)

        assertTrue(controls.canRefreshAccount)
        assertTrue(controls.canStartBrowserLogin)
        assertTrue(controls.canStartDeviceCodeLogin)
        assertTrue(controls.canRefreshUsage)
        assertFalse(controls.canOpenBrowser)
        assertFalse(controls.canLogout)
    }
}
