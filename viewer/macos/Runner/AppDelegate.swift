import Cocoa
import FlutterMacOS
import app_links
import multiview_desktop

@main
class AppDelegate: FlutterAppDelegate {

    public override func application(
        _ application: NSApplication,
        continue userActivity: NSUserActivity,
        restorationHandler: @escaping ([any NSUserActivityRestoring]) -> Void
    ) -> Bool {

        guard let url = AppLinks.shared.getUniversalLink(userActivity) else {
            return false
        }

        AppLinks.shared.handleLink(link: url.absoluteString)

        return false
    }

    override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return MultiviewDesktopPlugin.applicationShouldTerminateAfterLastWindowClosed()
    }

    override func applicationShouldHandleReopen(
        _ sender: NSApplication,
        hasVisibleWindows flag: Bool
    ) -> Bool {
        if MultiviewDesktopPlugin.applicationShouldHandleReopen(sender, hasVisibleWindows: flag) {
            return true
        }
        return super.applicationShouldHandleReopen(sender, hasVisibleWindows: flag)
    }

    override func applicationShouldTerminate(_ sender: NSApplication)
        -> NSApplication.TerminateReply
    {
        return MultiviewDesktopPlugin.applicationShouldTerminate(sender)
    }

    override func applicationDockMenu(_ sender: NSApplication) -> NSMenu? {
        return MultiviewDesktopPlugin.applicationDockMenu(sender)
    }

    override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
        return true
    }
}
