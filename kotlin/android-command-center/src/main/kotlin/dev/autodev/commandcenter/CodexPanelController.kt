package dev.autodev.commandcenter

interface CodexOperations {
    fun account(endpoint: String): CodexAccountInfo

    fun startBrowserLogin(endpoint: String): CodexLoginPrompt.Browser

    fun startDeviceCodeLogin(endpoint: String): CodexLoginPrompt.DeviceCode

    fun rateLimits(endpoint: String): CodexRateLimitInfo

    fun logout(endpoint: String)
}

class CodexPanelController(
    private val operations: CodexOperations,
) {
    fun refreshAccount(
        endpoint: String,
        state: CodexPanelState,
    ): CodexPanelState =
        runOperation(state, { operations.account(endpoint) }) {
            CodexPanelEvent.AccountLoaded(it)
        }

    fun startBrowserLogin(
        endpoint: String,
        state: CodexPanelState,
    ): CodexPanelState =
        runOperation(state, { operations.startBrowserLogin(endpoint) }) {
            CodexPanelEvent.LoginPromptReady(it)
        }

    fun startDeviceCodeLogin(
        endpoint: String,
        state: CodexPanelState,
    ): CodexPanelState =
        runOperation(state, { operations.startDeviceCodeLogin(endpoint) }) {
            CodexPanelEvent.LoginPromptReady(it)
        }

    fun refreshRateLimits(
        endpoint: String,
        state: CodexPanelState,
    ): CodexPanelState =
        runOperation(state, { operations.rateLimits(endpoint) }) {
            CodexPanelEvent.RateLimitsLoaded(it)
        }

    fun logout(
        endpoint: String,
        state: CodexPanelState,
    ): CodexPanelState =
        runOperation(state, { operations.logout(endpoint) }) {
            CodexPanelEvent.LoggedOut
        }

    private fun <T> runOperation(
        state: CodexPanelState,
        operation: () -> T,
        successEvent: (T) -> CodexPanelEvent,
    ): CodexPanelState {
        val loadingState = reduceCodexPanelState(state, CodexPanelEvent.Loading)
        return try {
            reduceCodexPanelState(loadingState, successEvent(operation()))
        } catch (error: CodexApiException) {
            reduceCodexPanelState(loadingState, CodexPanelEvent.Failed(error.errorCode))
        } catch (_: Exception) {
            reduceCodexPanelState(loadingState, CodexPanelEvent.Failed("network_error"))
        }
    }
}
