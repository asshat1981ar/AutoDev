package dev.autodev.platform

/**
 * Raised when a platform filesystem operation fails (missing path, I/O error,
 * or an unimplemented target).
 */
public class FileSystemException(message: String) : Exception(message)
