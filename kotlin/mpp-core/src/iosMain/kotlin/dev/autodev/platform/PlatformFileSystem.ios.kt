package dev.autodev.platform

import kotlinx.cinterop.ExperimentalForeignApi
import platform.Foundation.NSData
import platform.Foundation.NSFileManager
import platform.Foundation.NSString
import platform.Foundation.NSUTF8StringEncoding
import platform.Foundation.create
import platform.Foundation.dataUsingEncoding
import platform.Foundation.dataWithContentsOfFile
import platform.Foundation.fileExistsAtPath
import platform.Foundation.writeToFile
import platform.objc.autoreleasepool

/**
 * iOS / Kotlin-Native implementation of [PlatformFileSystem] backed by
 * [NSFileManager].
 *
 * Darwin objects are created inside [autoreleasepool] blocks so no temporary
 * objects outlive the call, matching the Kotlin-Native memory guidance for
 * Foundation API usage.
 */
@OptIn(ExperimentalForeignApi::class)
actual class PlatformFileSystem actual constructor() {
    actual fun readText(path: String): String =
        autoreleasepool {
            val manager = NSFileManager.defaultManager
            if (!manager.fileExistsAtPath(path)) {
                throw FileSystemException("file not found: $path")
            }
            val data =
                NSData.dataWithContentsOfFile(path)
                    ?: throw FileSystemException("read failed for $path")
            val str =
                NSString.create(data, NSUTF8StringEncoding)
                    ?: throw FileSystemException("decode failed for $path (not UTF-8)")
            str as String
        }

    actual fun writeText(
        path: String,
        content: String,
    ) {
        autoreleasepool {
            val nsString = content as NSString
            val data =
                nsString.dataUsingEncoding(NSUTF8StringEncoding)
                    ?: throw FileSystemException("encode failed for $path (not UTF-8)")
            val ok = data.writeToFile(path, atomically = true)
            if (!ok) {
                throw FileSystemException("write failed for $path")
            }
        }
    }

    actual fun exists(path: String): Boolean =
        autoreleasepool {
            NSFileManager.defaultManager.fileExistsAtPath(path)
        }
}
