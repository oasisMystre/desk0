#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
// EXTRA BEGIN
typedef struct DartCObject *WireSyncRust2DartDco;
typedef struct WireSyncRust2DartSse {
  uint8_t *ptr;
  int32_t len;
} WireSyncRust2DartSse;

typedef int64_t DartPort;
typedef bool (*DartPostCObjectFnType)(DartPort port_id, void *message);
void store_dart_post_cobject(DartPostCObjectFnType ptr);
// EXTRA END
typedef struct _Dart_Handle* Dart_Handle;

typedef struct wire_cst_list_prim_u_8_strict {
  uint8_t *ptr;
  int32_t len;
} wire_cst_list_prim_u_8_strict;

typedef struct wire_cst_sse_desktop_size {
  uint16_t width;
  uint16_t height;
} wire_cst_sse_desktop_size;

typedef struct wire_cst_record_u_32_u_32 {
  uint32_t field0;
  uint32_t field1;
} wire_cst_record_u_32_u_32;

typedef struct wire_cst_SseInputEvent_WheelRotations {
  bool is_vertical;
  int16_t rotation_units;
} wire_cst_SseInputEvent_WheelRotations;

typedef struct wire_cst_SseInputEvent_MouseButtonPressed {
  int32_t field0;
} wire_cst_SseInputEvent_MouseButtonPressed;

typedef struct wire_cst_SseInputEvent_MouseButtonReleased {
  int32_t field0;
} wire_cst_SseInputEvent_MouseButtonReleased;

typedef struct wire_cst_SseInputEvent_MousePosition {
  uint16_t x;
  uint16_t y;
} wire_cst_SseInputEvent_MousePosition;

typedef struct wire_cst_SseInputEvent_KeyPressed {
  uint8_t code;
  bool extended;
} wire_cst_SseInputEvent_KeyPressed;

typedef struct wire_cst_SseInputEvent_KeyReleased {
  uint8_t code;
  bool extended;
} wire_cst_SseInputEvent_KeyReleased;

typedef union SseInputEventKind {
  struct wire_cst_SseInputEvent_WheelRotations WheelRotations;
  struct wire_cst_SseInputEvent_MouseButtonPressed MouseButtonPressed;
  struct wire_cst_SseInputEvent_MouseButtonReleased MouseButtonReleased;
  struct wire_cst_SseInputEvent_MousePosition MousePosition;
  struct wire_cst_SseInputEvent_KeyPressed KeyPressed;
  struct wire_cst_SseInputEvent_KeyReleased KeyReleased;
} SseInputEventKind;

typedef struct wire_cst_sse_input_event {
  int32_t tag;
  union SseInputEventKind kind;
} wire_cst_sse_input_event;

typedef struct wire_cst_list_sse_input_event {
  struct wire_cst_sse_input_event *ptr;
  int32_t len;
} wire_cst_list_sse_input_event;

typedef struct wire_cst_SseIoInputEvent_Download {
  uint32_t id;
  struct wire_cst_list_prim_u_8_strict *uri;
  struct wire_cst_list_prim_u_8_strict *path;
} wire_cst_SseIoInputEvent_Download;

typedef union SseIoInputEventKind {
  struct wire_cst_SseIoInputEvent_Download Download;
} SseIoInputEventKind;

typedef struct wire_cst_sse_io_input_event {
  int32_t tag;
  union SseIoInputEventKind kind;
} wire_cst_sse_io_input_event;

typedef struct wire_cst_SseStaticChannelInputEvent_Io {
  struct wire_cst_sse_io_input_event *field0;
} wire_cst_SseStaticChannelInputEvent_Io;

typedef struct wire_cst_SseTerminalInputEvent_Create {
  uint32_t field0;
} wire_cst_SseTerminalInputEvent_Create;

typedef struct wire_cst_SseTerminalInputEvent_Exit {
  uint32_t field0;
} wire_cst_SseTerminalInputEvent_Exit;

typedef struct wire_cst_SseTerminalInputEvent_Data {
  uint32_t id;
  struct wire_cst_list_prim_u_8_strict *data;
  int32_t mode;
} wire_cst_SseTerminalInputEvent_Data;

typedef struct wire_cst_SseTerminalInputEvent_Resize {
  uint32_t id;
  uint16_t rows;
  uint16_t cols;
  uint16_t pw;
  uint16_t ph;
} wire_cst_SseTerminalInputEvent_Resize;

typedef union SseTerminalInputEventKind {
  struct wire_cst_SseTerminalInputEvent_Create Create;
  struct wire_cst_SseTerminalInputEvent_Exit Exit;
  struct wire_cst_SseTerminalInputEvent_Data Data;
  struct wire_cst_SseTerminalInputEvent_Resize Resize;
} SseTerminalInputEventKind;

typedef struct wire_cst_sse_terminal_input_event {
  int32_t tag;
  union SseTerminalInputEventKind kind;
} wire_cst_sse_terminal_input_event;

typedef struct wire_cst_SseStaticChannelInputEvent_Terminal {
  struct wire_cst_sse_terminal_input_event *field0;
} wire_cst_SseStaticChannelInputEvent_Terminal;

typedef union SseStaticChannelInputEventKind {
  struct wire_cst_SseStaticChannelInputEvent_Io Io;
  struct wire_cst_SseStaticChannelInputEvent_Terminal Terminal;
} SseStaticChannelInputEventKind;

typedef struct wire_cst_sse_static_channel_input_event {
  int32_t tag;
  union SseStaticChannelInputEventKind kind;
} wire_cst_sse_static_channel_input_event;

typedef struct wire_cst_list_sse_static_channel_input_event {
  struct wire_cst_sse_static_channel_input_event *ptr;
  int32_t len;
} wire_cst_list_sse_static_channel_input_event;

typedef struct wire_cst_SseDisconnectReason_Other {
  struct wire_cst_list_prim_u_8_strict *field0;
} wire_cst_SseDisconnectReason_Other;

typedef union SseDisconnectReasonKind {
  struct wire_cst_SseDisconnectReason_Other Other;
} SseDisconnectReasonKind;

typedef struct wire_cst_sse_disconnect_reason {
  int32_t tag;
  union SseDisconnectReasonKind kind;
} wire_cst_sse_disconnect_reason;

typedef struct wire_cst_SseFileDownloadStatus_Download {
  uint32_t id;
  uint64_t total_size;
} wire_cst_SseFileDownloadStatus_Download;

typedef struct wire_cst_SseFileDownloadStatus_Progress {
  uint32_t id;
  uint64_t total_size;
  uint64_t received;
} wire_cst_SseFileDownloadStatus_Progress;

typedef struct wire_cst_SseFileDownloadStatus_Completed {
  uint32_t id;
} wire_cst_SseFileDownloadStatus_Completed;

typedef union SseFileDownloadStatusKind {
  struct wire_cst_SseFileDownloadStatus_Download Download;
  struct wire_cst_SseFileDownloadStatus_Progress Progress;
  struct wire_cst_SseFileDownloadStatus_Completed Completed;
} SseFileDownloadStatusKind;

typedef struct wire_cst_sse_file_download_status {
  int32_t tag;
  union SseFileDownloadStatusKind kind;
} wire_cst_sse_file_download_status;

typedef struct wire_cst_SseTerminalError_IoFailed {
  int32_t field0;
} wire_cst_SseTerminalError_IoFailed;

typedef union SseTerminalErrorKind {
  struct wire_cst_SseTerminalError_IoFailed IoFailed;
} SseTerminalErrorKind;

typedef struct wire_cst_sse_terminal_error {
  int32_t tag;
  union SseTerminalErrorKind kind;
} wire_cst_sse_terminal_error;

typedef struct wire_cst_sse_monitor {
  int32_t left;
  int32_t top;
  int32_t right;
  int32_t bottom;
  uint32_t flags;
} wire_cst_sse_monitor;

typedef struct wire_cst_list_sse_monitor {
  struct wire_cst_sse_monitor *ptr;
  int32_t len;
} wire_cst_list_sse_monitor;

typedef struct wire_cst_SseClipboardEvent_File {
  struct wire_cst_sse_file_download_status *field0;
} wire_cst_SseClipboardEvent_File;

typedef union SseClipboardEventKind {
  struct wire_cst_SseClipboardEvent_File File;
} SseClipboardEventKind;

typedef struct wire_cst_sse_clipboard_event {
  int32_t tag;
  union SseClipboardEventKind kind;
} wire_cst_sse_clipboard_event;

typedef struct wire_cst_SseIoOutputEvent_DownloadStarted {
  uint32_t field0;
} wire_cst_SseIoOutputEvent_DownloadStarted;

typedef struct wire_cst_SseIoOutputEvent_DownloadProgress {
  uint32_t id;
  uint64_t downloaded;
  uint64_t *total;
} wire_cst_SseIoOutputEvent_DownloadProgress;

typedef struct wire_cst_SseIoOutputEvent_DownloadCompleted {
  uint32_t id;
  struct wire_cst_list_prim_u_8_strict *path;
} wire_cst_SseIoOutputEvent_DownloadCompleted;

typedef struct wire_cst_SseIoOutputEvent_DownloadFailed {
  uint32_t id;
  struct wire_cst_list_prim_u_8_strict *description;
} wire_cst_SseIoOutputEvent_DownloadFailed;

typedef union SseIoOutputEventKind {
  struct wire_cst_SseIoOutputEvent_DownloadStarted DownloadStarted;
  struct wire_cst_SseIoOutputEvent_DownloadProgress DownloadProgress;
  struct wire_cst_SseIoOutputEvent_DownloadCompleted DownloadCompleted;
  struct wire_cst_SseIoOutputEvent_DownloadFailed DownloadFailed;
} SseIoOutputEventKind;

typedef struct wire_cst_sse_io_output_event {
  int32_t tag;
  union SseIoOutputEventKind kind;
} wire_cst_sse_io_output_event;

typedef struct wire_cst_SseOutputEvent_PointerPosition {
  uint16_t x;
  uint16_t y;
} wire_cst_SseOutputEvent_PointerPosition;

typedef struct wire_cst_SseOutputEvent_PointerBitmap {
  uint16_t width;
  uint16_t height;
  uint16_t hotspot_x;
  uint16_t hotspot_y;
  struct wire_cst_list_prim_u_8_strict *bitmap_data;
} wire_cst_SseOutputEvent_PointerBitmap;

typedef struct wire_cst_SseOutputEvent_Resize {
  uint16_t width;
  uint16_t height;
} wire_cst_SseOutputEvent_Resize;

typedef struct wire_cst_SseOutputEvent_WindowingOrders {
  struct wire_cst_list_prim_u_8_strict *field0;
} wire_cst_SseOutputEvent_WindowingOrders;

typedef struct wire_cst_SseOutputEvent_MonitorLayout {
  struct wire_cst_list_sse_monitor *field0;
} wire_cst_SseOutputEvent_MonitorLayout;

typedef struct wire_cst_SseOutputEvent_TerminatedError {
  struct wire_cst_list_prim_u_8_strict *field0;
} wire_cst_SseOutputEvent_TerminatedError;

typedef struct wire_cst_SseOutputEvent_TerminatedSuccess {
  struct wire_cst_sse_disconnect_reason *field0;
} wire_cst_SseOutputEvent_TerminatedSuccess;

typedef struct wire_cst_SseOutputEvent_ConnectionFailure {
  struct wire_cst_list_prim_u_8_strict *field0;
} wire_cst_SseOutputEvent_ConnectionFailure;

typedef struct wire_cst_SseOutputEvent_AutoReconnecting {
  uint32_t disconnect_reason;
  uint32_t attempt;
  uint32_t maximum_attempts;
} wire_cst_SseOutputEvent_AutoReconnecting;

typedef union SseOutputEventKind {
  struct wire_cst_SseOutputEvent_PointerPosition PointerPosition;
  struct wire_cst_SseOutputEvent_PointerBitmap PointerBitmap;
  struct wire_cst_SseOutputEvent_Resize Resize;
  struct wire_cst_SseOutputEvent_WindowingOrders WindowingOrders;
  struct wire_cst_SseOutputEvent_MonitorLayout MonitorLayout;
  struct wire_cst_SseOutputEvent_TerminatedError TerminatedError;
  struct wire_cst_SseOutputEvent_TerminatedSuccess TerminatedSuccess;
  struct wire_cst_SseOutputEvent_ConnectionFailure ConnectionFailure;
  struct wire_cst_SseOutputEvent_AutoReconnecting AutoReconnecting;
} SseOutputEventKind;

typedef struct wire_cst_sse_output_event {
  int32_t tag;
  union SseOutputEventKind kind;
} wire_cst_sse_output_event;

typedef struct wire_cst_SseTerminalOutputEvent_Connected {
  uint32_t field0;
} wire_cst_SseTerminalOutputEvent_Connected;

typedef struct wire_cst_SseTerminalOutputEvent_Data {
  uint32_t id;
  struct wire_cst_list_prim_u_8_strict *data;
} wire_cst_SseTerminalOutputEvent_Data;

typedef struct wire_cst_SseTerminalOutputEvent_Exit {
  uint32_t id;
  int32_t code;
  struct wire_cst_list_prim_u_8_strict *description;
} wire_cst_SseTerminalOutputEvent_Exit;

typedef struct wire_cst_SseTerminalOutputEvent_Error {
  uint32_t id;
  struct wire_cst_sse_terminal_error *code;
  struct wire_cst_list_prim_u_8_strict *description;
} wire_cst_SseTerminalOutputEvent_Error;

typedef union SseTerminalOutputEventKind {
  struct wire_cst_SseTerminalOutputEvent_Connected Connected;
  struct wire_cst_SseTerminalOutputEvent_Data Data;
  struct wire_cst_SseTerminalOutputEvent_Exit Exit;
  struct wire_cst_SseTerminalOutputEvent_Error Error;
} SseTerminalOutputEventKind;

typedef struct wire_cst_sse_terminal_output_event {
  int32_t tag;
  union SseTerminalOutputEventKind kind;
} wire_cst_sse_terminal_output_event;

void frbgen_desk0_wire__crate__flutter__ViewerHandle_connect(int64_t port_, uintptr_t that);

WireSyncRust2DartDco frbgen_desk0_wire__crate__flutter__ViewerHandle_create_texture(uintptr_t that,
                                                                                    int64_t engine_handle);

WireSyncRust2DartDco frbgen_desk0_wire__crate__flutter__ViewerHandle_new(struct wire_cst_list_prim_u_8_strict *token,
                                                                         struct wire_cst_list_prim_u_8_strict *endpoint,
                                                                         struct wire_cst_sse_desktop_size *size);

void frbgen_desk0_wire__crate__flutter__ViewerHandle_on_clipboard_event(int64_t port_,
                                                                        uintptr_t that,
                                                                        struct wire_cst_list_prim_u_8_strict *events);

void frbgen_desk0_wire__crate__flutter__ViewerHandle_on_io_output_event_event(int64_t port_,
                                                                              uintptr_t that,
                                                                              struct wire_cst_list_prim_u_8_strict *events);

void frbgen_desk0_wire__crate__flutter__ViewerHandle_on_rdp_event(int64_t port_,
                                                                  uintptr_t that,
                                                                  struct wire_cst_list_prim_u_8_strict *events);

void frbgen_desk0_wire__crate__flutter__ViewerHandle_on_terminal_output_event_event(int64_t port_,
                                                                                    uintptr_t that,
                                                                                    struct wire_cst_list_prim_u_8_strict *events);

void frbgen_desk0_wire__crate__flutter__ViewerHandle_resize(int64_t port_,
                                                            uintptr_t that,
                                                            uint16_t width,
                                                            uint16_t height,
                                                            uint32_t scale_factor,
                                                            struct wire_cst_record_u_32_u_32 *physical_size);

void frbgen_desk0_wire__crate__flutter__ViewerHandle_send_input_events(int64_t port_,
                                                                       uintptr_t that,
                                                                       struct wire_cst_list_sse_input_event *events);

void frbgen_desk0_wire__crate__flutter__ViewerHandle_send_static_channel_input_events(int64_t port_,
                                                                                      uintptr_t that,
                                                                                      struct wire_cst_list_sse_static_channel_input_event *events);

void frbgen_desk0_rust_arc_increment_strong_count_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerViewerHandle(const void *ptr);

void frbgen_desk0_rust_arc_decrement_strong_count_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerViewerHandle(const void *ptr);

struct wire_cst_record_u_32_u_32 *frbgen_desk0_cst_new_box_autoadd_record_u_32_u_32(void);

struct wire_cst_sse_desktop_size *frbgen_desk0_cst_new_box_autoadd_sse_desktop_size(void);

struct wire_cst_sse_disconnect_reason *frbgen_desk0_cst_new_box_autoadd_sse_disconnect_reason(void);

struct wire_cst_sse_file_download_status *frbgen_desk0_cst_new_box_autoadd_sse_file_download_status(void);

struct wire_cst_sse_io_input_event *frbgen_desk0_cst_new_box_autoadd_sse_io_input_event(void);

struct wire_cst_sse_terminal_error *frbgen_desk0_cst_new_box_autoadd_sse_terminal_error(void);

struct wire_cst_sse_terminal_input_event *frbgen_desk0_cst_new_box_autoadd_sse_terminal_input_event(void);

uint64_t *frbgen_desk0_cst_new_box_autoadd_u_64(uint64_t value);

struct wire_cst_list_prim_u_8_strict *frbgen_desk0_cst_new_list_prim_u_8_strict(int32_t len);

struct wire_cst_list_sse_input_event *frbgen_desk0_cst_new_list_sse_input_event(int32_t len);

struct wire_cst_list_sse_monitor *frbgen_desk0_cst_new_list_sse_monitor(int32_t len);

struct wire_cst_list_sse_static_channel_input_event *frbgen_desk0_cst_new_list_sse_static_channel_input_event(int32_t len);
static int64_t dummy_method_to_enforce_bundling(void) {
    int64_t dummy_var = 0;
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_box_autoadd_record_u_32_u_32);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_box_autoadd_sse_desktop_size);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_box_autoadd_sse_disconnect_reason);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_box_autoadd_sse_file_download_status);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_box_autoadd_sse_io_input_event);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_box_autoadd_sse_terminal_error);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_box_autoadd_sse_terminal_input_event);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_box_autoadd_u_64);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_list_prim_u_8_strict);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_list_sse_input_event);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_list_sse_monitor);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_cst_new_list_sse_static_channel_input_event);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_rust_arc_decrement_strong_count_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerViewerHandle);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_rust_arc_increment_strong_count_RustOpaque_flutter_rust_bridgefor_generatedRustAutoOpaqueInnerViewerHandle);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_connect);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_create_texture);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_new);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_on_clipboard_event);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_on_io_output_event_event);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_on_rdp_event);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_on_terminal_output_event_event);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_resize);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_send_input_events);
    dummy_var ^= ((int64_t) (void*) frbgen_desk0_wire__crate__flutter__ViewerHandle_send_static_channel_input_events);
    dummy_var ^= ((int64_t) (void*) store_dart_post_cobject);
    return dummy_var;
}
