package dev.autodev.agents

import dev.autodev.codegraph.KotlinTreeSitterParser

class SemanticPatchSubAgent {
    private val parser = KotlinTreeSitterParser()

    fun analyze(source: String) = parser.parse(source)
}
