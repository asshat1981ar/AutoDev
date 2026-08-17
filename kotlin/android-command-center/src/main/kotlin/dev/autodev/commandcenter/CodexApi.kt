package dev.autodev.commandcenter

import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject

data class HttpRequestSpec(
    val method: String,
    val url: String,
)

data class HttpResult(
    val statusCode: Int,
    val body: String,
)

fun interface CommandCenterHttpTransport {
    fun execute(request: HttpRequestSpec): HttpResult
}

class OkHttpCommandCenterTransport(
    private val client: OkHttpClient = OkHttpClient(),
) : CommandCenterHttpTransport {
    override fun execute(request: HttpRequestSpec): HttpResult {
        val builder = Request.Builder().url(request.url)
        when (request.method) {
            "GET" -> builder.get()
            "POST" -> builder.post(ByteArray(0).toRequestBody(null))
            else -> throw IllegalArgumentException("unsupported HTTP method")
        }

        client.newCall(builder.build()).execute().use { response ->
            return HttpResult(
                statusCode = response.code,
                body = response.body?.string().orEmpty(),
            )
        }
    }
}

data class CodexAccountInfo(
    val authenticated: Boolean,
    val authMode: String?,
    val planType: String?,
)

sealed interface CodexLoginPrompt {
    data class Browser(
        val loginId: String,
        val authUrl: String,
    ) : CodexLoginPrompt

    data class DeviceCode(
        val loginId: String,
        val verificationUrl: String,
        val userCode: String,
    ) : CodexLoginPrompt
}

data class CodexRateLimitInfo(
    val limitId: String?,
    val planType: String?,
    val usedPercent: Int?,
    val resetsAt: Long?,
    val resetCreditsAvailable: Int?,
    val creditBalance: String?,
)

class CodexApiException(
    val statusCode: Int?,
    val errorCode: String,
) : RuntimeException(errorCode)

class CodexApi(
    private val transport: CommandCenterHttpTransport,
) {
    fun account(endpoint: String): CodexAccountInfo {
        val result = execute(endpoint, "GET", "/api/v1/codex/account")
        val json = parseJson(result)
        return CodexAccountInfo(
            authenticated = requiredBoolean(json, "authenticated", result.statusCode),
            authMode = optionalString(json, "auth_mode"),
            planType = optionalString(json, "plan_type"),
        )
    }

    fun startBrowserLogin(endpoint: String): CodexLoginPrompt.Browser {
        val result = execute(endpoint, "POST", "/api/v1/codex/login/browser")
        val json = parseJson(result)
        if (optionalString(json, "type") != "browser") {
            throw CodexApiException(result.statusCode, "invalid_response")
        }
        return CodexLoginPrompt.Browser(
            loginId = requiredString(json, "login_id", result.statusCode),
            authUrl = requiredString(json, "auth_url", result.statusCode),
        )
    }

    fun startDeviceCodeLogin(endpoint: String): CodexLoginPrompt.DeviceCode {
        val result = execute(endpoint, "POST", "/api/v1/codex/login/device-code")
        val json = parseJson(result)
        if (optionalString(json, "type") != "device_code") {
            throw CodexApiException(result.statusCode, "invalid_response")
        }
        return CodexLoginPrompt.DeviceCode(
            loginId = requiredString(json, "login_id", result.statusCode),
            verificationUrl = requiredString(json, "verification_url", result.statusCode),
            userCode = requiredString(json, "user_code", result.statusCode),
        )
    }

    fun rateLimits(endpoint: String): CodexRateLimitInfo {
        val result = execute(endpoint, "GET", "/api/v1/codex/rate-limits")
        val json = parseJson(result)
        val defaults = requiredObject(json, "default", result.statusCode)
        val primary = optionalObject(defaults, "primary")
        val credits = optionalObject(defaults, "credits")
        return CodexRateLimitInfo(
            limitId = optionalString(defaults, "limit_id"),
            planType = optionalString(defaults, "plan_type"),
            usedPercent = primary?.let { optionalInt(it, "used_percent") },
            resetsAt = primary?.let { optionalLong(it, "resets_at") },
            resetCreditsAvailable = optionalInt(json, "reset_credits_available"),
            creditBalance = credits?.let { optionalString(it, "balance") },
        )
    }

    fun logout(endpoint: String) {
        execute(endpoint, "POST", "/api/v1/codex/logout")
    }

    private fun execute(
        endpoint: String,
        method: String,
        path: String,
    ): HttpResult {
        val baseUrl = normalizeEndpoint(endpoint)
        val result = transport.execute(HttpRequestSpec(method, "$baseUrl$path"))
        if (result.statusCode !in 200..299) {
            throw apiError(result)
        }
        return result
    }

    private fun normalizeEndpoint(endpoint: String): String {
        val normalized = endpoint.trim().trimEnd('/')
        if (normalized.isEmpty()) {
            throw CodexApiException(null, "invalid_endpoint")
        }
        return normalized
    }

    private fun apiError(result: HttpResult): CodexApiException {
        val errorCode =
            runCatching {
                optionalString(JSONObject(result.body), "error")
            }.getOrNull()
                ?: "http_${result.statusCode}"
        return CodexApiException(result.statusCode, errorCode)
    }

    private fun parseJson(result: HttpResult): JSONObject =
        try {
            JSONObject(result.body)
        } catch (_: Exception) {
            throw CodexApiException(result.statusCode, "invalid_response")
        }

    private fun requiredString(
        json: JSONObject,
        key: String,
        statusCode: Int,
    ): String =
        optionalString(json, key)
            ?.takeIf { it.isNotBlank() }
            ?: throw CodexApiException(statusCode, "invalid_response")

    private fun requiredBoolean(
        json: JSONObject,
        key: String,
        statusCode: Int,
    ): Boolean =
        if (json.has(key) && !json.isNull(key)) {
            runCatching { json.getBoolean(key) }
                .getOrElse { throw CodexApiException(statusCode, "invalid_response") }
        } else {
            throw CodexApiException(statusCode, "invalid_response")
        }

    private fun requiredObject(
        json: JSONObject,
        key: String,
        statusCode: Int,
    ): JSONObject =
        optionalObject(json, key)
            ?: throw CodexApiException(statusCode, "invalid_response")

    private fun optionalObject(
        json: JSONObject,
        key: String,
    ): JSONObject? =
        if (json.has(key) && !json.isNull(key)) {
            json.optJSONObject(key)
        } else {
            null
        }

    private fun optionalString(
        json: JSONObject,
        key: String,
    ): String? =
        if (json.has(key) && !json.isNull(key)) {
            runCatching { json.getString(key) }.getOrNull()
        } else {
            null
        }

    private fun optionalInt(
        json: JSONObject,
        key: String,
    ): Int? =
        if (json.has(key) && !json.isNull(key)) {
            runCatching { json.getInt(key) }.getOrNull()
        } else {
            null
        }

    private fun optionalLong(
        json: JSONObject,
        key: String,
    ): Long? =
        if (json.has(key) && !json.isNull(key)) {
            runCatching { json.getLong(key) }.getOrNull()
        } else {
            null
        }
}
