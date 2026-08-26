package dev.autodev.server

import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty

/**
 * AutoDev server entry point.
 *
 * Starts a Ktor Netty server with the SSE streaming router mounted.
 * The port defaults to 8080 and can be overridden with the `AUTODEV_PORT`
 * environment variable.
 */
fun main() {
  val port = (System.getenv("AUTODEV_PORT") ?: "8080").toIntOrNull() ?: 8080
  val router = SseStreamingRouter()

  embeddedServer(Netty, port = port) {
    sseRoutes(router)
  }.start(wait = true)
}
