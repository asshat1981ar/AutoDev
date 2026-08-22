package dev.autodev.server

import io.ktor.client.request.get
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.TextContent
import io.ktor.http.contentType
import io.ktor.serialization.kotlinx.json.json
import io.ktor.server.application.install
import io.ktor.server.plugins.contentnegotiation.ContentNegotiation
import io.ktor.server.testing.testApplication
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.coroutines.flow.flowOf
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertNotNull
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

/**
 * Tests for the SSE streaming router and the bounded objective-enqueue
 * endpoint, using Ktor's in-process test host so no real port is bound.
 */
class SseStreamingRouterTest {
  @Test
  fun `health endpoint returns ok`() = testApplication {
    application {
      install(ContentNegotiation) { json() }
      sseRoutes(SseStreamingRouter(flowOf("ready")))
    }
    val response = client.get("/health")
    assertEquals(HttpStatusCode.OK, response.status)
    val body = response.bodyAsText()
    assertTrue(body.contains("ok"), "expected health body to contain 'ok', got: $body")
  }

  @Test
  fun `events endpoint advertises event-stream content type`() = testApplication {
    application {
      install(ContentNegotiation) { json() }
      sseRoutes(SseStreamingRouter(flowOf("ready")))
    }
    val response = client.get("/events")
    val contentType = response.headers[HttpHeaders.ContentType]
    assertNotNull(contentType, "Content-Type header must be present")
    assertTrue(
      contentType!!.startsWith("text/event-stream"),
      "expected text/event-stream content type, got: $contentType",
    )
  }

  @Test
  fun `events endpoint emits data id and keepalive frames`() = testApplication {
    application {
      install(ContentNegotiation) { json() }
      sseRoutes(SseStreamingRouter(flowOf("alpha", "beta")))
    }
    val body = client.get("/events").bodyAsText()
    assertTrue(body.contains("data: alpha"), "expected 'data: alpha' frame, got: $body")
    assertTrue(body.contains("data: beta"), "expected 'data: beta' frame, got: $body")
    assertTrue(body.contains("retry: 5000"), "expected 'retry:' hint, got: $body")
    assertTrue(body.contains("id: 1"), "expected sequential 'id:' frame, got: $body")
    assertTrue(body.contains(": keepalive"), "expected ': keepalive' comment frame, got: $body")
  }

  @Test
  fun `objective enqueue accepts a bounded payload`() = testApplication {
    application {
      install(ContentNegotiation) { json() }
      sseRoutes(SseStreamingRouter(flowOf("ready")))
    }
    val response = client.post("/api/v1/objectives") {
      // Use a JsonObject as the body. Do not call contentType() explicitly:
      // with install(ContentNegotiation) { json() } on the client, the
      // JSON plugin sets the Content-Type header automatically based on
      // the body's concrete type. Calling contentType() separately can
      // produce an HttpSend IllegalStateException (Content-Length/body
      // length mismatch). The production server uses call.receiveText()
      // and accepts any non-empty body that is not oversized.
      setBody(JsonObject(mapOf("description" to JsonPrimitive("hello"))))
    }
    assertEquals(HttpStatusCode.Accepted, response.status)
    val body = response.bodyAsText()
    assertTrue(body.contains("queued"), "expected 'queued' in body, got: $body")
    assertTrue(body.contains("queue_size"), "expected 'queue_size' in body, got: $body")
  }

  @Test
  fun `objective enqueue rejects empty payload`() = testApplication {
    application {
      install(ContentNegotiation) { json() }
      sseRoutes(SseStreamingRouter(flowOf("ready")))
    }
    val response = client.post("/api/v1/objectives") {
      contentType(ContentType.Application.Json)
      setBody("")
    }
    assertEquals(HttpStatusCode.BadRequest, response.status)
  }

  @Test
  fun `objective enqueue rejects oversized payload`() = testApplication {
    application {
      install(ContentNegotiation) { json() }
      sseRoutes(SseStreamingRouter(flowOf("ready")))
    }
    val oversized = "x".repeat(SseStreamingRouter.MAX_OBJECTIVE_BYTES + 1)
    val response = client.post("/api/v1/objectives") {
      contentType(ContentType.Application.Json)
      setBody(oversized)
    }
    assertEquals(HttpStatusCode.PayloadTooLarge, response.status)
  }
}
