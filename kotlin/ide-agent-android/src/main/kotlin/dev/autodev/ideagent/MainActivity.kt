package dev.autodev.ideagent

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    IdeAgentApp()
                }
            }
        }
    }
}

@Composable
private fun IdeAgentApp(backend: ExecutionBackend = PreviewExecutionBackend()) {
    var state by remember { mutableStateOf(IdeAgentState(backendName = backend.name)) }
    var selectedTab by remember { mutableIntStateOf(0) }
    val scope = rememberCoroutineScope()

    Scaffold(
        bottomBar = {
            NavigationBar {
                listOf("Plan", "Changes", "Build", "Review").forEachIndexed { index, label ->
                    NavigationBarItem(
                        selected = selectedTab == index,
                        onClick = { selectedTab = index },
                        icon = { Text((index + 1).toString()) },
                        label = { Text(label) },
                    )
                }
            }
        },
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .padding(16.dp)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text("IDE Agent", style = MaterialTheme.typography.headlineMedium, fontWeight = FontWeight.Bold)
            Text("Android control plane for the Koog development-agent pipeline")

            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    Text("Workspace", fontWeight = FontWeight.Bold)
                    Text(state.projectName)
                    Text("Runner: ${state.backendName}")
                }
            }

            OutlinedTextField(
                value = state.task,
                onValueChange = { state = state.copy(task = it) },
                modifier = Modifier.fillMaxWidth(),
                label = { Text("What should we build?") },
                minLines = 3,
            )

            Button(
                onClick = {
                    scope.launch {
                        val task = state.task.trim()
                        if (task.isEmpty()) return@launch
                        state = state.copy(
                            steps = AgentStage.entries.map { AgentStep(it) },
                            isRunning = true,
                        )
                        val result = backend.execute(task) { stage, detail ->
                            state = state
                                .markRunning(stage, detail)
                                .markSucceeded(stage, "$detail complete")
                        }
                        state = state.copy(
                            plan = result.plan,
                            changes = result.changes,
                            buildOutput = result.buildOutput,
                            review = result.review,
                        ).finish()
                    }
                },
                enabled = state.task.isNotBlank() && !state.isRunning,
            ) {
                Text(if (state.isRunning) "Running…" else "Run pipeline")
            }

            HorizontalDivider()
            Text("Agent pipeline", fontWeight = FontWeight.Bold)
            state.steps.forEach { step -> AgentStepCard(step) }

            Spacer(Modifier.height(4.dp))
            ResultCard(
                title = listOf("Plan", "Changes", "Build", "Review")[selectedTab],
                content = when (selectedTab) {
                    0 -> state.plan
                    1 -> state.changes
                    2 -> state.buildOutput
                    else -> state.review
                },
            )
        }
    }
}

@Composable
private fun AgentStepCard(step: AgentStep) {
    val status = when (step.status) {
        StageStatus.WAITING -> "○"
        StageStatus.RUNNING -> "…"
        StageStatus.SUCCEEDED -> "✓"
        StageStatus.FAILED -> "!"
    }
    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth().padding(12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column {
                Text(step.stage.label, fontWeight = FontWeight.SemiBold)
                Text(step.detail, style = MaterialTheme.typography.bodySmall)
            }
            Text(status, fontWeight = FontWeight.Bold)
        }
    }
}

@Composable
private fun ResultCard(title: String, content: String) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(title, fontWeight = FontWeight.Bold)
            Text(content)
        }
    }
}
