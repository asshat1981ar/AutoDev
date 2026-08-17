package dev.autodev.commandcenter

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

class CodexApiNetworkTest {
    @Test
    fun transport_failures_become_sanitized_recoverable_errors() {
        val transport =
            CommandCenterHttpTransport {
                error("socket-detail-must-not-escape")
            }
        val api = CodexApi(transport)

        val failure =
            assertFailsWith<CodexApiException> {
                api.account("http://server")
            }

        assertNull(failure.statusCode)
        assertEquals("network_error", failure.errorCode)
        assertNull(failure.cause)
    }
}
