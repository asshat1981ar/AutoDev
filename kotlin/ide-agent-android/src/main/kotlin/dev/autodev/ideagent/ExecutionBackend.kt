package dev.autodev.ideagent

import kotlinx.coroutines.delay

data class ExecutionResult(
    val plan: String,
    val changes: String,
    val buildOutput: String,
    val review: String,
)

interface ExecutionBackend {
    val name: String

    suspend fun execute(
        task: String,
        onStage: suspend (AgentStage, String) -> Unit,
    ): ExecutionResult
}

class PreviewExecutionBackend : ExecutionBackend {
    override val name: String = "Preview backend"

    override suspend fun execute(
        task: String,
        onStage: suspend (AgentStage, String) -> Unit,
    ): ExecutionResult {
        require(task.isNotBlank()) { "Task must not be blank" }

        onStage(AgentStage.PLANNER, "Decomposing task")
        delay(250)
        onStage(AgentStage.CODER, "Preparing bounded changes")
        delay(250)
        onStage(AgentStage.BUILDER, "Simulating Gradle verification")
        delay(250)
        onStage(AgentStage.REVIEWER, "Reviewing acceptance criteria")
        delay(250)

        return ExecutionResult(
            plan = "1. Inspect project context\n2. Apply the smallest safe change\n3. Build and test\n4. Review the diff",
            changes = "Preview mode: no files were modified.\nRequested task: $task",
            buildOutput = "PREVIEW PASS\nA real backend will replace this with Gradle output.",
            review = "Preview review passed. Connect a Termux or remote backend for executable workspace changes.",
        )
    }
}
