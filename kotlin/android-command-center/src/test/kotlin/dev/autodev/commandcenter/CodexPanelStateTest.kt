package dev.autodev.commandcenter

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertSame
import kotlin.test.assertTrue

class CodexPanelStateTest {
    private val account =
        CodexAccountInfo(
            authenticated = true,
            authMode = "chatgpt",
            planType = "plus",
        )
    private val limits =
        CodexRateLimitInfo(
            limitId = "codex",
            planType = "plus",
            usedPercent = 42,
            resetsAt = 1_800_000_000L,
            resetCreditsAvailable = 2,
            creditBalance = "7.50",
        )
    private val prompt =
        CodexLoginPrompt.DeviceCode(
            loginId = "login-1",
            verificationUrl = "https://auth.openai.com/codex/device",
            userCode = "ABCD-1234",
        )

    @Test
    fun loading_sets_busy_and_clears_only_the_error() {
        val existing =
            CodexPanelState(
                account = account,
                rateLimits = limits,
                loginPrompt = prompt,
                errorCode = "network_error",
            )

        val next = reduceCodexPanelState(existing, CodexPanelEvent.Loading)

        assertTrue(next.busy)
        assertNull(next.errorCode)
        assertSame(account, next.account)
        assertSame(limits, next.rateLimits)
        assertSame(prompt, next.loginPrompt)
    }

    @Test
    fun successful_account_refresh_clears_login_prompt_after_authentication() {
        val existing = CodexPanelState(loginPrompt = prompt, busy = true, errorCode = "old")

        val next = reduceCodexPanelState(existing, CodexPanelEvent.AccountLoaded(account))

        assertFalse(next.busy)
        assertNull(next.errorCode)
        assertSame(account, next.account)
        assertNull(next.loginPrompt)
    }

    @Test
    fun login_and_usage_success_update_only_their_owned_fields() {
        val login =
            reduceCodexPanelState(
                CodexPanelState(account = account, rateLimits = limits, busy = true),
                CodexPanelEvent.LoginPromptReady(prompt),
            )
        assertSame(account, login.account)
        assertSame(limits, login.rateLimits)
        assertSame(prompt, login.loginPrompt)
        assertFalse(login.busy)

        val usage = reduceCodexPanelState(login, CodexPanelEvent.RateLimitsLoaded(limits))
        assertSame(account, usage.account)
        assertSame(prompt, usage.loginPrompt)
        assertSame(limits, usage.rateLimits)
        assertFalse(usage.busy)
        assertNull(usage.errorCode)
    }

    @Test
    fun failures_preserve_last_good_data_and_expose_only_stable_error_code() {
        val existing =
            CodexPanelState(
                account = account,
                rateLimits = limits,
                loginPrompt = prompt,
                busy = true,
            )

        val next = reduceCodexPanelState(existing, CodexPanelEvent.Failed("network_error"))

        assertFalse(next.busy)
        assertEquals("network_error", next.errorCode)
        assertSame(account, next.account)
        assertSame(limits, next.rateLimits)
        assertSame(prompt, next.loginPrompt)
    }

    @Test
    fun logout_clears_all_subscription_session_view_state() {
        val existing =
            CodexPanelState(
                account = account,
                rateLimits = limits,
                loginPrompt = prompt,
                busy = true,
                errorCode = "old",
            )

        val next = reduceCodexPanelState(existing, CodexPanelEvent.LoggedOut)

        assertFalse(next.busy)
        assertNull(next.account)
        assertNull(next.rateLimits)
        assertNull(next.loginPrompt)
        assertNull(next.errorCode)
    }
}
