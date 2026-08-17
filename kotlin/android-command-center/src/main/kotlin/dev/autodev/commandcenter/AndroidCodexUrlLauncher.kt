package dev.autodev.commandcenter

import android.content.Context
import android.content.Intent
import android.net.Uri

class AndroidCodexUrlLauncher(context: Context) : CodexUrlLauncher {
    private val launchContext = context.applicationContext ?: context

    override fun open(url: String) {
        val intent =
            Intent(Intent.ACTION_VIEW, Uri.parse(url))
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        if (intent.resolveActivity(launchContext.packageManager) != null) {
            launchContext.startActivity(intent)
        }
    }
}
