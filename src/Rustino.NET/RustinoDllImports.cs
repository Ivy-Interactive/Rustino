using System.Runtime.InteropServices;

namespace Rustino.NET;

// Callback delegate types matching Rust extern "C" fn signatures
[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int ClosingCallback(IntPtr context);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void VoidContextCallback(IntPtr context);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void SizeCallback(IntPtr context, int width, int height);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void PointCallback(IntPtr context, int x, int y);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void IntCallback(IntPtr context, int value);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void StringCallback(IntPtr context, IntPtr message);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate void PageLoadCallback(IntPtr context, int eventType, IntPtr url);

[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
internal delegate int NavigationCallback(IntPtr context, IntPtr url);

internal static class RustinoDllImports
{
    private const string Lib = NativeLibraryResolver.LibName;

    // --- Lifecycle ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr rustino_ctor(ref RustinoNativeParameters parameters);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_dtor(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_wait_for_exit(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_close(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_free_string(IntPtr s);

    // --- Notifications ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int rustino_show_notification(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string title,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string body,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? icon,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? appId);

    // --- Dual-mode setters ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_title(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string title);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_size(IntPtr instance, int width, int height);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_resizable(IntPtr instance, int resizable);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_topmost(IntPtr instance, int topmost);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_icon_file(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string path);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_center(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_navigate_to_url(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string url);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_navigate_to_string(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string content);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_background_color(
        IntPtr instance, byte r, byte g, byte b, byte a);

    // --- Pre-run only setters ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_use_os_default_size(IntPtr instance, int useDefault);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_devtools_enabled(IntPtr instance, int enabled);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_clipboard_enabled(IntPtr instance, int enabled);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_ignore_cert_errors(IntPtr instance, int enabled);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_web_security_enabled(IntPtr instance, int enabled);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_transparent(IntPtr instance, int transparent);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_decorations(IntPtr instance, int decorated);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_user_agent(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string userAgent);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_user_data_folder(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string path);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_media_autoplay(IntPtr instance, int enabled);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_zoom_hotkeys(IntPtr instance, int enabled);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_add_init_script(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string js);

    // --- Window state ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_minimized(IntPtr instance, int minimized);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_maximized(IntPtr instance, int maximized);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_fullscreen(IntPtr instance, int fullscreen);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_visible(IntPtr instance, int visible);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_focus(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_position(IntPtr instance, int x, int y);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_min_size(IntPtr instance, int width, int height);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_max_size(IntPtr instance, int width, int height);

    // --- State queries ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int rustino_is_minimized(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int rustino_is_maximized(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern int rustino_is_fullscreen(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_get_position(IntPtr instance, out int x, out int y);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_get_size(IntPtr instance, out int width, out int height);

    // --- WebView operations ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_evaluate_script(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string js);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_send_web_message(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string message);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_zoom(IntPtr instance, double factor);

    // --- Dialogs ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr rustino_show_open_file_dialog(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? title,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? defaultPath,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? filters,
        int multiSelect);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr rustino_show_save_file_dialog(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? title,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? defaultPath,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? filters);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr rustino_show_select_folder_dialog(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? title,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? defaultPath,
        int multiSelect);

    // --- Monitors ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr rustino_get_monitors(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr rustino_get_current_monitor(IntPtr instance);

    // --- Badge ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_badge_count(IntPtr instance, int count);

    // --- Menus & Tray ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_menu(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string json);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_remove_menu(IntPtr instance);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_show_context_menu(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string json,
        double x,
        double y);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_tray_icon(
        IntPtr instance,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string iconPath,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? tooltip,
        [MarshalAs(UnmanagedType.LPUTF8Str)] string? menuJson);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_remove_tray_icon(IntPtr instance);

    // --- Callback registration ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_callback_context(IntPtr instance, IntPtr context);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_closing_handler(IntPtr instance, ClosingCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_closed_handler(IntPtr instance, VoidContextCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_resized_handler(IntPtr instance, SizeCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_moved_handler(IntPtr instance, PointCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_focus_changed_handler(IntPtr instance, IntCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_web_message_received_handler(IntPtr instance, StringCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_page_load_handler(IntPtr instance, PageLoadCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_navigation_handler(IntPtr instance, NavigationCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_menu_event_handler(IntPtr instance, StringCallback handler);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_set_tray_icon_event_handler(IntPtr instance, VoidContextCallback handler);

    // --- Splashscreen ---

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern IntPtr rustino_splash_create(
        [MarshalAs(UnmanagedType.LPUTF8Str)] string imagePath,
        int width,
        int height);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_splash_close(IntPtr splash);

    [DllImport(Lib, CallingConvention = CallingConvention.Cdecl)]
    internal static extern void rustino_splash_dtor(IntPtr splash);
}
