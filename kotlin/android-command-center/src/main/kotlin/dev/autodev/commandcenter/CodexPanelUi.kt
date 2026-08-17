package dev.autodev.commandcenter

import java.net.URI

data class CodexPanelUiModel(
    val authStatus: String,
    val planType: String?,
    val usedPercent: Int?,
    val resetsAt: Long?,
    val resetCreditsAvailable: Int?,
    val creditBalance: String?,
    val browserAuthUrl: String?,
    val deviceVerificationUrl: String?,
    val deviceUserCode: String?,
    val busy: Boolean,
    val errorCode: String?,
)

fun codexPanelUiModel(state: CodexPanelState): CodexPanelUiModel {
    val browserPrompt = state.loginPrompt as? CodexLoginPrompt.Browser
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
        browserAuthUrl = browserPrompt?.authUrl,
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
    fun open(prompt: CodexLoginPrompt?): Boolean =
        openUrl((prompt as? CodexLoginPrompt.Browser)?.authUrl)

    fun openUrl(url: String?): Boolean {
        val candidate = url ?: return false
        val uri = runCatching { URI(candidate) }.getOrNull() ?: return false
        if (!uri.scheme.equals("https", ignoreCase = true) || uri.host.isNullOrBlank()) return false
        launcher.open(candidate)
        return true
    }
}
