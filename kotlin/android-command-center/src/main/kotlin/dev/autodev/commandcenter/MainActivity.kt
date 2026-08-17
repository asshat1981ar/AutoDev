package dev.autodev.commandcenter

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import okhttp3.Call
import okhttp3.OkHttpClient
import okhttp3.Request
import java.util.concurrent.TimeUnit

private const val DEFAULT_SERVER = "http://10.0.2.2:8080"
private const val MAX_EVENTS = 200

data class CommandCenterState(
    val endpoint: String = DEFAULT_SERVER,
    val connected: Boolean = false,
    val status: String = "Disconnected",
    val events: List<String> = emptyList(),
)

class CommandCenterViewModel(
    private val codexController: CodexPanelController =
        CodexPanelController(CodexApi(OkHttpCommandCenterTransport())),
    private val codexDispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val client =
        OkHttpClient.Builder()
            .readTimeout(0, TimeUnit.MILLISECONDS)
            .build()

    private val mutableState = MutableStateFlow(CommandCenterState())
    val state: StateFlow<CommandCenterState> = mutableState.asStateFlow()

    private val mutableCodexState = MutableStateFlow(CodexPanelState())
    val codexState: StateFlow<CodexPanelState> = mutableCodexState.asStateFlow()

    private var streamJob: Job? = null
    private var activeCall: Call? = null

    fun connect(rawEndpoint: String) {
        val endpoint = rawEndpoint.trim().trimEnd('/')
        if (endpoint.isEmpty()) return

        cancelStream()
        mutableState.update {
            it.copy(endpoint = endpoint, connected = false, status = "Connecting…", events = emptyList())
        }

        streamJob =
            viewModelScope.launch(Dispatchers.IO) {
                val request = Request.Builder().url("$endpoint/events").get().build()
                val call = client.newCall(request)
                activeCall = call
                try {
                    call.execute().use { response ->
                        if (!response.isSuccessful) error("HTTP ${response.code}")
                        val source = response.body?.source() ?: error("Empty response body")
                        mutableState.update { it.copy(connected = true, status = "Connected") }

                        while (!source.exhausted()) {
                            val line = source.readUtf8Line() ?: break
                            if (!line.startsWith("data:")) continue
                            val event = line.removePrefix("data:").trim()
                            if (event.isEmpty()) continue
                            mutableState.update { current ->
                                current.copy(events = (listOf(event) + current.events).take(MAX_EVENTS))
                            }
                        }
                        mutableState.update { it.copy(connected = false, status = "Stream closed") }
                    }
                } catch (failure: Exception) {
                    if (!call.isCanceled()) {
                        mutableState.update {
                            it.copy(connected = false, status = failure.message ?: "Connection failed")
                        }
                    }
                } finally {
                    if (activeCall === call) activeCall = null
                }
            }
    }

    fun disconnect() {
        cancelStream()
        mutableState.update { it.copy(connected = false, status = "Disconnected") }
    }

    fun refreshCodexAccount(endpoint: String) {
        updateCodexState(endpoint, codexController::refreshAccount)
    }

    fun startCodexBrowserLogin(endpoint: String) {
        updateCodexState(endpoint, codexController::startBrowserLogin)
    }

    fun startCodexDeviceCodeLogin(endpoint: String) {
        updateCodexState(endpoint, codexController::startDeviceCodeLogin)
    }

    fun refreshCodexRateLimits(endpoint: String) {
        updateCodexState(endpoint, codexController::refreshRateLimits)
    }

    fun logoutCodex(endpoint: String) {
        updateCodexState(endpoint, codexController::logout)
    }

    private fun updateCodexState(
        endpoint: String,
        operation: (String, CodexPanelState) -> CodexPanelState,
    ) {
        viewModelScope.launch(codexDispatcher) {
            mutableCodexState.value = operation(endpoint, mutableCodexState.value)
        }
    }

    private fun cancelStream() {
        activeCall?.cancel()
        activeCall = null
        streamJob?.cancel()
        streamJob = null
    }

    override fun onCleared() {
        cancelStream()
        client.dispatcher.executorService.shutdown()
        super.onCleared()
    }
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    commandCenterScreen()
                }
            }
        }
    }
}

@Composable
private fun commandCenterScreen(viewModel: CommandCenterViewModel = viewModel()) {
    val state by viewModel.state.collectAsState()
    var endpoint by remember(state.endpoint) { mutableStateOf(state.endpoint) }

    Column(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("AutoDev Command Center", style = MaterialTheme.typography.headlineSmall)
        Text(
            "Observer-only Android control plane. Agent authority remains inside ForgeCore.",
            style = MaterialTheme.typography.bodyMedium,
        )

        OutlinedTextField(
            value = endpoint,
            onValueChange = { endpoint = it },
            label = { Text("AutoDev server") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )

        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(onClick = { viewModel.connect(endpoint) }) { Text("Connect") }
            Button(onClick = viewModel::disconnect, enabled = state.connected) { Text("Disconnect") }
        }

        Card(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.padding(12.dp)) {
                Text("Status", style = MaterialTheme.typography.labelLarge)
                Text(state.status)
            }
        }

        Text("Live events", style = MaterialTheme.typography.titleMedium)
        LazyColumn(
            modifier = Modifier.fillMaxWidth().weight(1f),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            if (state.events.isEmpty()) {
                item { Text("No events received yet.") }
            } else {
                items(state.events) { event ->
                    Card(modifier = Modifier.fillMaxWidth()) {
                        Text(event, modifier = Modifier.padding(12.dp))
                    }
                }
            }
        }
    }
}
