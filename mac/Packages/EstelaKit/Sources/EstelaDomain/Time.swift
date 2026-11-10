public struct Time: Hashable, Comparable, Sendable {
    public static let zero = Time(nanoseconds: 0)

    private static let nanosecondsPerSecond: Double = 1_000_000_000

    public let nanoseconds: Int64

    public init(nanoseconds: Int64) {
        self.nanoseconds = nanoseconds
    }

    public init(seconds: Double) {
        self.nanoseconds = Int64((seconds * Self.nanosecondsPerSecond).rounded())
    }

    public var seconds: Double {
        Double(nanoseconds) / Self.nanosecondsPerSecond
    }

    public static func < (lhs: Time, rhs: Time) -> Bool {
        lhs.nanoseconds < rhs.nanoseconds
    }

    public static func + (lhs: Time, rhs: Time) -> Time {
        Time(nanoseconds: lhs.nanoseconds + rhs.nanoseconds)
    }

    public static func - (lhs: Time, rhs: Time) -> Time {
        Time(nanoseconds: lhs.nanoseconds - rhs.nanoseconds)
    }
}
