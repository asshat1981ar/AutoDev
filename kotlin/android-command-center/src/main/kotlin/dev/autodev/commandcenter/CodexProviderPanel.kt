package dev.autodev.commandcenter

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp

@Composable
internal fun codexProviderPanel(
    state: CodexPanelState,
    onRefreshAccount: () -> Unit,
    onStartBrowserLogin: () -> Unit,
    onStartDeviceCodeLogin: () -> Unit,
    onRefreshUsage: () -> Unit,
    onLogout: () -> Unit,
) {
    val context = LocalContext.current
    val browserAction =
        remember(context) {
            CodexBrowserAction(AndroidCodexUrlLauncher(context))
        }
    val model = codexPanelUiModel(state)
    val controls = codexPanelControls(model)

    Card(modifier = Modifier.fillMaxWidth()) {
        Column(
            modifier = Modifier.padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text("Codex / ChatGPT subscription", style = MaterialTheme.typography.titleMedium)
            Text("Status: ${model.authStatus}")
            model.planType?.let { Text("Plan: $it") }
            model.usedPercent?.let { Text("Usage: $it%") }
            model.resetsAt?.let { Text("Resets at: $it") }
            model.resetCreditsAvailable?.let { Text("Reset credits: $it") }
            model.creditBalance?.let { Text("Credit balance: $it") }
            if (model.busy) Text("Working…")
            model.errorCode?.let { Text("Error: $it") }

            model.deviceVerificationUrl?.let { Text("Device verification: $it") }
            model.deviceUserCode?.let { Text("Device code: $it") }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Button(
                    onClick = onRefreshAccount,
                    enabled = controls.canRefreshAccount,
                    modifier = Modifier.weight(1f),
                ) {
                    Text("Account")
                }
                Button(
                    onClick = onRefreshUsage,
                    enabled = controls.canRefreshUsage,
                    modifier = Modifier.weight(1f),
                ) {
                    Text("Usage")
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Button(
                    onClick = onStartBrowserLogin,
                    enabled = controls.canStartBrowserLogin,
                    modifier = Modifier.weight(1f),
                ) {
                    Text("Browser sign-in")
                }
                Button(
                    onClick = onStartDeviceCodeLogin,
                    enabled = controls.canStartDeviceCodeLogin,
                    modifier = Modifier.weight(1f),
                ) {
                    Text("Device code")
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Button(
                    onClick = { browserAction.openUrl(model.browserAuthUrl) },
                    enabled = controls.canOpenBrowser,
                    modifier = Modifier.weight(1f),
                ) {
                    Text("Open sign-in")
                }
                Button(
                    onClick = onLogout,
                    enabled = controls.canLogout,
                    modifier = Modifier.weight(1f),
                ) {
                    Text("Logout")
                }
            }

            Text(
                "OAuth credentials remain on the AutoDev server; this app receives status and login prompts only.",
                style = MaterialTheme.typography.bodySmall,
            )
        }
    }
}
