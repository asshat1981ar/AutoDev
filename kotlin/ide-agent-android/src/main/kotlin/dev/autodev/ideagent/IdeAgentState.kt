package dev.autodev.ideagent

enum class AgentStage(val label: String) {
    PLANNER("Planner"),
    CODER("Coder"),
    BUILDER("Builder"),
    REVIEWER("Reviewer"),
}

enum class StageStatus {
    WAITING,
    RUNNING,
    SUCCEEDED,
    FAILED,
}

data class AgentStep(
    val stage: AgentStage,
    val status: StageStatus = StageStatus.WAITING,
    val detail: String = "Waiting",
)

data class IdeAgentState(
    val projectName: String = "No project selected",
    val task: String = "",
    val backendName: String = "Preview backend",
    val steps: List<AgentStep> = AgentStage.entries.map { AgentStep(it) },
    val plan: String = "No plan yet.",
    val changes: String = "No changes yet.",
    val buildOutput: String = "No build yet.",
    val review: String = "No review yet.",
    val isRunning: Boolean = false,
)

fun IdeAgentState.markRunning(stage: AgentStage, detail: String): IdeAgentState =
    copy(
        isRunning = true,
        steps = steps.map {
            if (it.stage == stage) it.copy(status = StageStatus.RUNNING, detail = detail) else it
        },
    )

fun IdeAgentState.markSucceeded(stage: AgentStage, detail: String): IdeAgentState =
    copy(
        steps = steps.map {
            if (it.stage == stage) it.copy(status = StageStatus.SUCCEEDED, detail = detail) else it
        },
    )

fun IdeAgentState.finish(): IdeAgentState = copy(isRunning = false)
