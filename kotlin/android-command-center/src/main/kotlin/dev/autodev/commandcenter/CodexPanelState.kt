package dev.autodev.commandcenter

data class CodexPanelState(
    val account: CodexAccountInfo? = null,
    val rateLimits: CodexRateLimitInfo? = null,
    val loginPrompt: CodexLoginPrompt? = null,
    val busy: Boolean = false,
    val errorCode: String? = null,
)

sealed interface CodexPanelEvent {
    data object Loading : CodexPanelEvent

    data class AccountLoaded(
        val account: CodexAccountInfo,
    ) : CodexPanelEvent

    data class LoginPromptReady(
        val prompt: CodexLoginPrompt,
    ) : CodexPanelEvent

    data class RateLimitsLoaded(
        val rateLimits: CodexRateLimitInfo,
    ) : CodexPanelEvent

    data class Failed(
        val errorCode: String,
    ) : CodexPanelEvent

    data object LoggedOut : CodexPanelEvent
}

fun reduceCodexPanelState(
    state: CodexPanelState,
    event: CodexPanelEvent,
): CodexPanelState =
    when (event) {
        CodexPanelEvent.Loading ->
            state.copy(
                busy = true,
                errorCode = null,
            )

        is CodexPanelEvent.AccountLoaded ->
            state.copy(
                account = event.account,
                loginPrompt = if (event.account.authenticated) null else state.loginPrompt,
                busy = false,
                errorCode = null,
            )

        is CodexPanelEvent.LoginPromptReady ->
            state.copy(
                loginPrompt = event.prompt,
                busy = false,
                errorCode = null,
            )

        is CodexPanelEvent.RateLimitsLoaded ->
            state.copy(
                rateLimits = event.rateLimits,
                busy = false,
                errorCode = null,
            )

        is CodexPanelEvent.Failed ->
            state.copy(
                busy = false,
                errorCode = event.errorCode,
            )

        CodexPanelEvent.LoggedOut -> CodexPanelState()
    }
