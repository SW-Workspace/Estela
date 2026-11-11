@testable import EstelaData
import Foundation
import Testing

struct ProjectBundleLayoutTests {
    private let layout = ProjectBundleLayout(root: URL(filePath: "/tmp/Demo.estela", directoryHint: .isDirectory))

    @Test func placesManifestAtRoot() {
        #expect(layout.manifest.path() == "/tmp/Demo.estela/project.json")
    }

    @Test func placesRecordingsInMediaFolder() {
        #expect(layout.screenRecording.path() == "/tmp/Demo.estela/media/screen.mp4")
        #expect(layout.microphoneRecording.path() == "/tmp/Demo.estela/media/mic.m4a")
        #expect(layout.events.path() == "/tmp/Demo.estela/media/events.jsonl")
    }

    @Test func placesThumbnailsFolderAtRoot() {
        #expect(layout.thumbnails.path() == "/tmp/Demo.estela/thumbnails/")
    }
}
