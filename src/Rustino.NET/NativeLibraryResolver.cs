using System.Reflection;
using System.Runtime.InteropServices;

namespace Rustino.NET;

internal static class NativeLibraryResolver
{
    internal const string LibName = "rustino_native";
    private static IntPtr _cachedHandle = IntPtr.Zero;

    static NativeLibraryResolver()
    {
        NativeLibrary.SetDllImportResolver(typeof(NativeLibraryResolver).Assembly, Resolve);
    }

    internal static void EnsureRegistered()
    {
        // Forces the static constructor to run
    }

    private static IntPtr Resolve(string libraryName, Assembly assembly, DllImportSearchPath? searchPath)
    {
        if (libraryName != LibName)
            return IntPtr.Zero;

        if (_cachedHandle != IntPtr.Zero)
            return _cachedHandle;

        if (NativeLibrary.TryLoad(libraryName, assembly, searchPath, out var handle))
            return _cachedHandle = handle;

        var libFileName = GetLibraryFileName();

        var baseDir = AppDomain.CurrentDomain.BaseDirectory;
        var basePath = Path.Combine(baseDir, libFileName);
        if (NativeLibrary.TryLoad(basePath, out handle))
            return _cachedHandle = handle;

        var rid = GetRuntimeIdentifier();
        var ridPath = Path.Combine(baseDir, "runtimes", rid, "native", libFileName);
        if (NativeLibrary.TryLoad(ridPath, out handle))
            return _cachedHandle = handle;

        throw new DllNotFoundException(
            $"Could not load native library '{libFileName}' for {rid}. " +
            $"Searched: '{basePath}', '{ridPath}'. " +
            "Ensure the Rustino native library is built (cargo build --release) and placed in the output directory.");
    }

    private static string GetLibraryFileName()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            return "rustino_native.dll";
        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            return "librustino_native.dylib";
        return "librustino_native.so";
    }

    private static string GetRuntimeIdentifier()
    {
        var arch = RuntimeInformation.OSArchitecture switch
        {
            Architecture.X64 => "x64",
            Architecture.Arm64 => "arm64",
            _ => "x64"
        };

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            return $"win-{arch}";
        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            return $"osx-{arch}";
        return $"linux-{arch}";
    }
}
