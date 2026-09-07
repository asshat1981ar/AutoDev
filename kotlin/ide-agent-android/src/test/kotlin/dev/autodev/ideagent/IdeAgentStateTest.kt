package dev.autodev.ideagent

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class IdeAgentStateTest {
    @Test
    fun pipelineStartsWithAllAgentsWaiting() {
        val state = IdeAgentState()
        assertEquals(4, state.steps.size)
        assertTrue(state.steps.all { it.status == StageStatus.WAITING })
        assertFalse(state.isRunning)
    }

    @Test
    fun stageTransitionsAreIsolated() {
        val state = IdeAgentState()
            .markRunning(AgentStage.CODER, "Editing")
            .markSucceeded(AgentStage.CODER, "Done")

        assertEquals(StageStatus.SUCCEEDED, state.steps.first { it.stage == AgentStage.CODER }.status)
        assertTrue(state.steps.filterNot { it.stage == AgentStage.CODER }.all { it.status == StageStatus.WAITING })
        assertTrue(state.isRunning)
        assertFalse(state.finish().isRunning)
    }
}
