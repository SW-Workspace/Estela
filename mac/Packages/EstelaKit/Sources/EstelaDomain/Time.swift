public struct Time: Hashable, Comparable, Sendable {
    public static let zero = Time(nanoseconds: 0)

    private static let nanosecondsPerSecond: Double = 1_000_000_000
    private static let representableNanoseconds = Double(Int64.min)..<Double(Int64.max)

    public let nanoseconds: Int64

    public init(nanoseconds: Int64) {
        self.nanoseconds = nanoseconds
    }

    public init?(seconds: Double) {
        let nanoseconds = (seconds * Self.nanosecondsPerSecond).rounded()
        guard Self.representableNanoseconds.contains(nanoseconds) else { return nil }
        self.nanoseconds = Int64(nanoseconds)
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
