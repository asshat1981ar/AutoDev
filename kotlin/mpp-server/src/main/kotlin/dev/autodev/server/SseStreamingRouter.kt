package dev.autodev.server

import io.ktor.http.ContentType
import io.ktor.http.withCharset
import io.ktor.server.application.Application
import io.ktor.server.application.call
import io.ktor.server.response.respond
import io.ktor.server.response.respondTextWriter
import io.ktor.server.routing.get
import io.ktor.server.routing.routing
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.flow

/**
 * Wires Server-Sent Events (SSE) streaming endpoints onto a Ktor
 * [Application].
 *
 * Ktor 2.3.x has no dedicated SSE plugin artifact; SSE is implemented by
 * streaming `text/event-stream` chunks via a non-blocking [Flow], so the
 * server thread is never parked waiting for the next event.
 *
 * Endpoints:
 *  - `GET /health`  — plain liveness probe.
 *  - `GET /events` — SSE stream of [eventFlow], one `data:` frame per emission.
 */
public class SseStreamingRouter(
  private val eventFlow: Flow<String> = defaultEvents(),
) {
  /**
   * Install routes onto [application].
   */
  public fun routes(application: Application) {
    application.routing {
      get("/health") {
        call.respond(mapOf("status" to "ok"))
      }
      get("/events") {
        call.respondTextWriter(
          contentType = ContentType.Text.EventStream.withCharset(Charsets.UTF_8),
        ) {
          eventFlow.collect { event ->
            write("data: $event\n\n")
            flush()
          }
        }
      }
    }
  }

  public companion object {
    /**
     * A default heartbeat event flow used when none is supplied.
     */
    public fun defaultEvents(): Flow<String> =
      flow {
        var n = 0
        while (true) {
          emit("ping ${n++}")
          delay(1000)
        }
      }
  }
}

/**
 * Reusable convenience to attach [SseStreamingRouter] from an [Application].
 */
public fun Application.sseRoutes(router: SseStreamingRouter) {
  router.routes(this)
}
