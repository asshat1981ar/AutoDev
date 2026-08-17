package dev.autodev.commandcenter

import java.net.URI

data class CodexPanelUiModel(
    val authStatus: String,
    val planType: String?,
    val usedPercent: Int?,
    val resetsAt: Long?,
    val resetCreditsAvailable: Int?,
    val creditBalance: String?,
    val deviceVerificationUrl: String?,
    val deviceUserCode: String?,
    val busy: Boolean,
    val errorCode: String?,
)

fun codexPanelUiModel(state: CodexPanelState): CodexPanelUiModel {
    val devicePrompt = state.loginPrompt as? CodexLoginPrompt.DeviceCode
    return CodexPanelUiModel(
        authStatus =
            when (state.account?.authenticated) {
                true -> "Authenticated"
                false -> "Not authenticated"
                null -> "Unknown"
            },
        planType = state.account?.planType ?: state.rateLimits?.planType,
        usedPercent = state.rateLimits?.usedPercent,
        resetsAt = state.rateLimits?.resetsAt,
        resetCreditsAvailable = state.rateLimits?.resetCreditsAvailable,
        creditBalance = state.rateLimits?.creditBalance,
        deviceVerificationUrl = devicePrompt?.verificationUrl,
        deviceUserCode = devicePrompt?.userCode,
        busy = state.busy,
        errorCode = state.errorCode,
    )
}

fun interface CodexUrlLauncher {
    fun open(url: String)
}

class CodexBrowserAction(
    private val launcher: CodexUrlLauncher,
) {
    fun open(prompt: CodexLoginPrompt?): Boolean {
        val browserPrompt = prompt as? CodexLoginPrompt.Browser ?: return false
        val uri = runCatching { URI(browserPrompt.authUrl) }.getOrNull() ?: return false
        if (!uri.scheme.equals("https", ignoreCase = true) || uri.host.isNullOrBlank()) return false
        launcher.open(browserPrompt.authUrl)
        return true
    }
}
