import Foundation

public struct ProjectBundleLayout: Sendable, Equatable {
    public static let fileExtension = "estela"

    public let root: URL

    public init(root: URL) {
        self.root = root
    }

    public var manifest: URL { root.appending(path: "project.json") }
    public var media: URL { root.appending(path: "media", directoryHint: .isDirectory) }
    public var screenRecording: URL { media.appending(path: "screen.mp4") }
    public var microphoneRecording: URL { media.appending(path: "mic.m4a") }
    public var events: URL { media.appending(path: "events.jsonl") }
    public var thumbnails: URL { root.appending(path: "thumbnails", directoryHint: .isDirectory) }
}
