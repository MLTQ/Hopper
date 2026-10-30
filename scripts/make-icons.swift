import AppKit
import Foundation

// Run from the repository root. The mark is drawn directly; no external artwork or fonts.
let size = 1024
let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
NSColor(red: 0.19, green: 0.34, blue: 0.26, alpha: 1).setFill()
NSBezierPath(rect: NSRect(x: 0, y: 0, width: size, height: size)).fill()
let mark = NSBezierPath()
mark.move(to: NSPoint(x: 260, y: 235)); mark.line(to: NSPoint(x: 260, y: 790))
mark.line(to: NSPoint(x: 398, y: 790)); mark.line(to: NSPoint(x: 398, y: 570))
mark.curve(to: NSPoint(x: 757, y: 425), controlPoint1: NSPoint(x: 615, y: 726), controlPoint2: NSPoint(x: 757, y: 609))
mark.line(to: NSPoint(x: 757, y: 235)); mark.line(to: NSPoint(x: 617, y: 235)); mark.line(to: NSPoint(x: 617, y: 425))
mark.curve(to: NSPoint(x: 398, y: 425), controlPoint1: NSPoint(x: 617, y: 572), controlPoint2: NSPoint(x: 398, y: 571))
mark.line(to: NSPoint(x: 398, y: 235)); mark.close()
NSColor(red: 0.96, green: 0.95, blue: 0.86, alpha: 1).setFill(); mark.fill()
NSColor(red: 0.91, green: 0.60, blue: 0.35, alpha: 1).setFill()
NSBezierPath(ovalIn: NSRect(x: 670, y: 713, width: 124, height: 124)).fill()
NSGraphicsContext.restoreGraphicsState()
let data = bitmap.representation(using: .png, properties: [:])!
let destinations = ["apps/desktop/icons/icon.png", "apps/ios/Hopper/Assets.xcassets/AppIcon.appiconset/icon.png"]
for path in destinations {
    let url = URL(fileURLWithPath: path)
    try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
    try data.write(to: url)
}
