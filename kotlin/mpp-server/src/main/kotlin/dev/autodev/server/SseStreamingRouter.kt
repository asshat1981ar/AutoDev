package dev.autodev.server

import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.withCharset
import io.ktor.server.application.Application
import io.ktor.server.application.call
import io.ktor.server.request.receiveText
import io.ktor.server.response.respond
import io.ktor.server.response.respondTextWriter
import io.ktor.server.routing.get
import io.ktor.server.routing.post
import io.ktor.server.routing.routing
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Wires Server-Sent Events (SSE) streaming endpoints and the bounded
 * objective-enqueue endpoint onto a Ktor [Application].
 *
 * Ktor 2.3.x has no dedicated SSE plugin artifact; SSE is implemented by
 * streaming `text/event-stream` chunks via a non-blocking [Flow], so the
 * server thread is never parked waiting for the next event.
 *
 * Endpoints:
 *  - `GET /health`  — plain liveness probe.
 *  - `GET /events` — SSE stream of [eventFlow], one `data:` frame per emission.
 *  - `POST /api/v1/objectives` — bounded objective enqueue. Mirrors the
 *    read-only observer's permitted write path; payloads are capped at
 *    [MAX_OBJECTIVE_BYTES] and the queue is bounded to
 *    [MAX_OBJECTIVE_QUEUE].
 */
public class SseStreamingRouter(
    private val eventFlow: Flow<String> = defaultEvents(),
) {
    private val objectiveLock = Mutex()
    private val objectiveQueue: ArrayDeque<String> = ArrayDeque()

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
                    // SSE handshake frames: id and retry hint help clients
                    // reconnect from the last delivered event id.
                    write("retry: 5000\n")
                    write("id: 0\n\n")
                    var seq = 0
                    eventFlow.collect { event ->
                        seq += 1
                        write("id: $seq\n")
                        write("data: $event\n\n")
                        flush()
                        // Keepalive comment so idle proxies do not close the
                        // connection while the upstream flow is quiet.
                        write(": keepalive\n\n")
                        flush()
                    }
                }
            }
            post("/api/v1/objectives") {
                val payload = call.receiveText()
                if (payload.isEmpty()) {
                    call.respond(HttpStatusCode.BadRequest, mapOf("error" to "empty payload"))
                    return@post
                }
                if (payload.length > MAX_OBJECTIVE_BYTES) {
                    call.respond(
                        HttpStatusCode.PayloadTooLarge,
                        mapOf("error" to "payload exceeds $MAX_OBJECTIVE_BYTES bytes"),
                    )
                    return@post
                }
                val accepted = objectiveLock.withLock {
                    if (objectiveQueue.size >= MAX_OBJECTIVE_QUEUE) {
                        false
                    } else {
                        objectiveQueue.addLast(payload)
                        true
                    }
                }
                if (accepted) {
                    call.respond(
                        HttpStatusCode.Accepted,
                        mapOf("status" to "queued", "queue_size" to objectiveQueue.size),
                    )
                } else {
                    call.respond(
                        HttpStatusCode.ServiceUnavailable,
                        mapOf("error" to "objective queue full"),
                    )
                }
            }
        }
    }

    public companion object {
        /**
         * Maximum size of a single objective payload (UTF-16 code units).
         */
        public const val MAX_OBJECTIVE_BYTES: Int = 64 * 1024

        /**
         * Maximum number of objectives held in the in-process queue.
         */
        public const val MAX_OBJECTIVE_QUEUE: Int = 256

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
