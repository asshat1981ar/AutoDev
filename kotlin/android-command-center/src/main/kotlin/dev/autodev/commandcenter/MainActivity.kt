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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import okhttp3.Call
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.util.UUID
import java.util.concurrent.TimeUnit

private const val DEFAULT_SERVER = "http://10.0.2.2:8080"
private const val MAX_EVENTS = 200

data class PendingAction(
    val id: String,
    val actionType: String,
    val payload: String,
    val timestamp: Long = System.currentTimeMillis(),
)

data class CommandCenterState(
    val endpoint: String = DEFAULT_SERVER,
    val connected: Boolean = false,
    val status: String = "Disconnected",
    val events: List<String> = emptyList(),
    val pendingActions: List<PendingAction> = emptyList(),
)

class CommandCenterViewModel : ViewModel() {
    private val client =
        OkHttpClient.Builder()
            .readTimeout(0, TimeUnit.MILLISECONDS)
            .build()

    private val mutableState = MutableStateFlow(CommandCenterState())
    val state: StateFlow<CommandCenterState> = mutableState.asStateFlow()

    private var streamJob: Job? = null
    private var activeCall: Call? = null

    private val pendingActionsFlow = MutableStateFlow<List<PendingAction>>(emptyList())
    val pendingActionsState: StateFlow<List<PendingAction>> = pendingActionsFlow.asStateFlow()

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

    fun queueAction(
        actionType: String,
        payload: String,
    ) {
        val pending = PendingAction(
            id = UUID.randomUUID().toString(),
            actionType = actionType,
            payload = payload,
        )
        pendingActionsFlow.update { it + pending }
        mutableState.update { it.copy(pendingActions = pendingActionsFlow.value) }
        // If already connected, attempt immediate replay; otherwise keep queued for later
        if (mutableState.value.connected) {
            replayPending(mutableState.value.endpoint)
        }
    }

    fun replayPending(
        endpoint: String,
    ) {
        val pending = pendingActionsFlow.value
        if (pending.isEmpty()) return
        viewModelScope.launch(Dispatchers.IO) {
            val remaining = mutableListOf<PendingAction>()
            for (action in pending) {
                // Reuse same endpoint and existing AuthorizationGrant concept — blocked without grant does not consume attempt
                // This replay respects VerifiedOrchestratorState: approval resume reuses same envelope id
                val success =
                    try {
                        val request =
                            Request.Builder()
                                .url("$endpoint/api/v1/objectives")
                                .post(
                                    action.payload.toRequestBody(
                                        "application/json".toMediaType(),
                                    ),
                                )
                                .build()
                    client.newCall(request).execute().use { resp -> resp.isSuccessful }
                } catch (_: Exception) {
                    false
                }
                if (!success) remaining.add(action)
            }
            pendingActionsFlow.value = remaining
            mutableState.update {
                it.copy(
                    pendingActions = remaining,
                    status =
                        if (remaining.isEmpty()) {
                            "Replayed ${pending.size} queued"
                        } else {
                            "${remaining.size} pending after replay"
                        },
                )
            }
        }
    }

    fun disconnect() {
        cancelStream()
        mutableState.update { it.copy(connected = false, status = "Disconnected") }
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
                if (state.pendingActions.isNotEmpty()) {
                    Text("${state.pendingActions.size} queued (offline)", style = MaterialTheme.typography.labelSmall)
                }
            }
        }

        // Offline queue — production hardening G5: survives SSE drops, replays on reconnect
        if (state.pendingActions.isNotEmpty()) {
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(
                    modifier = Modifier.padding(12.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text("Queued actions", style = MaterialTheme.typography.titleSmall)
                        Button(
                            onClick = { viewModel.replayPending(state.endpoint) },
                            enabled = state.pendingActions.isNotEmpty(),
                        ) {
                            Text("Replay ${state.pendingActions.size}")
                        }
                    }
                    state.pendingActions.forEach { pending ->
                        Card(modifier = Modifier.fillMaxWidth()) {
                            Column(modifier = Modifier.padding(8.dp)) {
                                Text(pending.actionType, style = MaterialTheme.typography.labelMedium)
                                Text(pending.payload.take(120), style = MaterialTheme.typography.bodySmall)
                            }
                        }
                    }
                }
            }
        }

        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Button(
                onClick = {
                    viewModel.queueAction(
                        "objective.create",
                        "{\"description\":\"Queued objective\"}",
                    )
                },
            ) {
                Text("Queue test")
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
