using System.Runtime.InteropServices;

namespace Rustino.NET;

[StructLayout(LayoutKind.Sequential)]
internal struct RustinoNativeParameters
{
    public IntPtr Title;
    public IntPtr IconFile;
    public int Width;
    public int Height;
    public int CenterOnInitialize;
    public int UseOsDefaultSize;
    public int Resizable;
    public int Topmost;
    public int DevToolsEnabled;
    public int ClipboardEnabled;
    public int IgnoreCertificateErrors;
    public int WebSecurityEnabled;
    public int LogVerbosity;
}
