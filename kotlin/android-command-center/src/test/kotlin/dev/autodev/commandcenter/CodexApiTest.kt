package dev.autodev.commandcenter

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlin.test.assertNull

private class FakeHttpTransport(
    responses: List<HttpResult>,
) : CommandCenterHttpTransport {
    val requests = mutableListOf<HttpRequestSpec>()
    private val pending = ArrayDeque(responses)

    override fun execute(request: HttpRequestSpec): HttpResult {
        requests += request
        return pending.removeFirst()
    }
}

class CodexApiTest {
    @Test
    fun account_uses_safe_fields_and_normalizes_endpoint() {
        val transport =
            FakeHttpTransport(
                listOf(
                    HttpResult(
                        200,
                        """
                        {
                          "authenticated": true,
                          "auth_mode": "chatgpt",
                          "plan_type": "plus",
                          "access_token": "must-not-escape"
                        }
                        """.trimIndent(),
                    ),
                ),
            )
        val api = CodexApi(transport)

        val account = api.account(" http://10.0.2.2:8080/ ")

        assertEquals(
            HttpRequestSpec("GET", "http://10.0.2.2:8080/api/v1/codex/account"),
            transport.requests.single(),
        )
        assertEquals(true, account.authenticated)
        assertEquals("chatgpt", account.authMode)
        assertEquals("plus", account.planType)
    }

    @Test
    fun browser_and_device_code_login_map_only_user_facing_metadata() {
        val transport =
            FakeHttpTransport(
                listOf(
                    HttpResult(
                        200,
                        """{"type":"browser","login_id":"login-1","auth_url":"https://chatgpt.com/auth"}""",
                    ),
                    HttpResult(
                        200,
                        """
                        {
                          "type": "device_code",
                          "login_id": "login-2",
                          "verification_url": "https://auth.openai.com/codex/device",
                          "user_code": "ABCD-1234"
                        }
                        """.trimIndent(),
                    ),
                ),
            )
        val api = CodexApi(transport)

        val browser = api.startBrowserLogin("http://server")
        val device = api.startDeviceCodeLogin("http://server")

        assertIs<CodexLoginPrompt.Browser>(browser)
        assertEquals("login-1", browser.loginId)
        assertEquals("https://chatgpt.com/auth", browser.authUrl)
        assertIs<CodexLoginPrompt.DeviceCode>(device)
        assertEquals("login-2", device.loginId)
        assertEquals("https://auth.openai.com/codex/device", device.verificationUrl)
        assertEquals("ABCD-1234", device.userCode)
        assertEquals(
            listOf(
                HttpRequestSpec("POST", "http://server/api/v1/codex/login/browser"),
                HttpRequestSpec("POST", "http://server/api/v1/codex/login/device-code"),
            ),
            transport.requests,
        )
    }

    @Test
    fun rate_limits_project_only_usage_fields_needed_by_ui() {
        val transport =
            FakeHttpTransport(
                listOf(
                    HttpResult(
                        200,
                        """
                        {
                          "default": {
                            "limit_id": "codex",
                            "limit_name": "Codex",
                            "primary": {
                              "used_percent": 42,
                              "window_duration_mins": 300,
                              "resets_at": 1800000000
                            },
                            "secondary": null,
                            "credits": {
                              "has_credits": true,
                              "unlimited": false,
                              "balance": "7.50"
                            },
                            "plan_type": "plus",
                            "reached_type": null
                          },
                          "by_limit_id": {},
                          "reset_credits_available": 2
                        }
                        """.trimIndent(),
                    ),
                ),
            )
        val api = CodexApi(transport)

        val limits = api.rateLimits("http://server/")

        assertEquals("codex", limits.limitId)
        assertEquals("plus", limits.planType)
        assertEquals(42, limits.usedPercent)
        assertEquals(1_800_000_000L, limits.resetsAt)
        assertEquals(2, limits.resetCreditsAvailable)
        assertEquals("7.50", limits.creditBalance)
    }

    @Test
    fun logout_accepts_empty_204_response() {
        val transport = FakeHttpTransport(listOf(HttpResult(204, "")))
        val api = CodexApi(transport)

        api.logout("http://server")

        assertEquals(
            HttpRequestSpec("POST", "http://server/api/v1/codex/logout"),
            transport.requests.single(),
        )
    }

    @Test
    fun sanitized_server_errors_become_recoverable_api_errors() {
        val transport =
            FakeHttpTransport(
                listOf(HttpResult(503, """{"error":"codex_provider_unavailable"}""")),
            )
        val api = CodexApi(transport)

        val failure =
            assertFailsWith<CodexApiException> {
                api.account("http://server")
            }

        assertEquals(503, failure.statusCode)
        assertEquals("codex_provider_unavailable", failure.errorCode)
        assertNull(failure.cause)
    }
}
