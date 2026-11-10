@testable import EstelaDomain
import Testing

struct TimeTests {
    @Test func convertsSecondsToNanoseconds() {
        #expect(Time(seconds: 1.5)?.nanoseconds == 1_500_000_000)
    }

    @Test func roundsFractionalNanoseconds() {
        #expect(Time(seconds: 0.000_000_000_6)?.nanoseconds == 1)
    }

    @Test(arguments: [Double.nan, .infinity, -.infinity, 1e12])
    func rejectsUnrepresentableSeconds(seconds: Double) {
        #expect(Time(seconds: seconds) == nil)
    }

    @Test func convertsNanosecondsToSeconds() {
        #expect(Time(nanoseconds: 250_000_000).seconds == 0.25)
    }

    @Test func addsAndSubtracts() {
        let start = Time(nanoseconds: 100)
        let duration = Time(nanoseconds: 40)
        #expect(start + duration == Time(nanoseconds: 140))
        #expect(start - duration == Time(nanoseconds: 60))
    }

    @Test func ordersByNanoseconds() {
        #expect(Time.zero < Time(nanoseconds: 1))
        #expect([Time(nanoseconds: 3), .zero, Time(nanoseconds: 2)].sorted().first == .zero)
    }
}
