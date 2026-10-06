import Cocoa
import FlutterMacOS
import multiview_desktop

class MainFlutterWindow: NSWindow {
    override func awakeFromNib() {
        super.awakeFromNib()

        let engine = FlutterEngine(
            name: "main_flutter_engine",
            project: nil,
            allowHeadlessExecution: true
        )
        MultiviewDesktopPlugin.prepareEngine(engine, window: self)

        let flutterViewController = FlutterViewController(
            engine: engine,
            nibName: nil,
            bundle: nil
        )
        self.contentViewController = flutterViewController

        if let flutterView = flutterViewController.view.subviews.first {
            flutterView.wantsLayer = true
            flutterView.layer?.isOpaque = false
            flutterView.layer?.backgroundColor = .clear
        }
        flutterViewController.view.wantsLayer = true
        flutterViewController.view.layer?.isOpaque = false
        flutterViewController.view.layer?.backgroundColor = .clear

        if let contentView = self.contentView {
            let glass = NSVisualEffectView(frame: contentView.bounds)
            glass.material = .hudWindow
            glass.blendingMode = .behindWindow
            glass.state = .active
            glass.autoresizingMask = [.width, .height]
            contentView.addSubview(
                glass, positioned: .below, relativeTo: flutterViewController.view)

            // --- native corner radius, lost because isOpaque = false ---
            contentView.wantsLayer = true
            contentView.layer?.cornerRadius = 10.0
            contentView.layer?.masksToBounds = true
            if #available(macOS 11.0, *) {
                contentView.layer?.cornerCurve = .continuous
            }
        }

        self.isOpaque = false
        self.backgroundColor = .clear
        self.hasShadow = true  // shadow is what sells the rounded-corner look

        // --- standard titlebar height/inset, lost without a toolbar ---
        let toolbar = NSToolbar(identifier: "MainToolbar")
        toolbar.showsBaselineSeparator = false
        self.toolbar = toolbar
        self.toolbarStyle = .unifiedCompact  // or .unified for taller variant

        self.titlebarAppearsTransparent = true
        self.titleVisibility = .hidden
        self.styleMask.insert(.fullSizeContentView)

        setupMethodChannel(flutterViewController)

        RegisterGeneratedPlugins(registry: flutterViewController)
    }

    private func setupMethodChannel(_ flutterViewController: FlutterViewController) {
        let channel = FlutterMethodChannel(
            name: "com.desk0.viewer/channels",
            binaryMessenger: flutterViewController.engine.binaryMessenger
        )

        channel.setMethodCallHandler { [weak self] call, result in
            guard let self else {
                result(
                    FlutterError(
                        code: "WINDOW_UNAVAILABLE",
                        message: "Main window is unavailable",
                        details: nil
                    )
                )
                return
            }

            switch call.method {
            case "getTrafficLightFrame":
                result(self.getTrafficLightFrame())

            default:
                result(FlutterMethodNotImplemented)
            }
        }
    }

    private func getTrafficLightFrame() -> [String: Double] {
        guard
            let contentView = self.contentView,
            let closeButton = self.standardWindowButton(.closeButton)
        else {
            return [
                "x": 0,
                "y": 0,
                "width": 0,
                "height": 0,
                "buttonDiameter": 0,
            ]
        }

        let buttonFrame =
            closeButton.superview?.convert(
                closeButton.frame,
                to: contentView
            ) ?? closeButton.frame

        let flutterY = contentView.bounds.height - buttonFrame.maxY

        return [
            "x": buttonFrame.minX,
            "y": flutterY,
            "width": buttonFrame.width,
            "height": buttonFrame.height,
            "buttonDiameter": min(buttonFrame.width, buttonFrame.height),
        ]
    }
}
