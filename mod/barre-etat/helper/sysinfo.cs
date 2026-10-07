// Compteurs bruts pour la seconde ligne de la barre : la mod calcule les
// pourcentages par differences entre deux appels, comme l'ancien statusline.exe.
// Sortie : total dispo idle kernel user disque_idle disque_query
using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

static class SysInfo {
  [StructLayout(LayoutKind.Sequential)]
  struct MemStatus {
    public uint Length, Load;
    public ulong Total, Avail, TotalPage, AvailPage, TotalVirtual, AvailVirtual, AvailExtended;
  }

  [DllImport("kernel32.dll")] static extern bool GlobalMemoryStatusEx(ref MemStatus m);
  [DllImport("kernel32.dll")] static extern bool GetSystemTimes(out long idle, out long kernel, out long user);
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
  static extern SafeFileHandle CreateFileW(string name, uint access, uint share, IntPtr sa, uint disposition, uint flags, IntPtr template);
  [DllImport("kernel32.dll", SetLastError = true)]
  static extern bool DeviceIoControl(SafeFileHandle h, uint code, IntPtr input, uint inputSize, byte[] output, uint outputSize, out uint returned, IntPtr overlapped);

  const uint IOCTL_DISK_PERFORMANCE = 0x70020;

  static void Main(string[] args) {
    var m = new MemStatus();
    m.Length = (uint)Marshal.SizeOf(typeof(MemStatus));
    GlobalMemoryStatusEx(ref m);
    long idle, kernel, user;
    GetSystemTimes(out idle, out kernel, out user);
    string disk = "- -";
    string drive = args.Length > 0 ? args[0].TrimEnd('\\') : "C:";
    // Acces 0 : l'IOCTL est FILE_ANY_ACCESS, pas besoin des droits administrateur
    using (var h = CreateFileW("\\\\.\\" + drive, 0, 3, IntPtr.Zero, 3, 0, IntPtr.Zero)) {
      if (!h.IsInvalid) {
        var buf = new byte[88];
        uint n;
        if (DeviceIoControl(h, IOCTL_DISK_PERFORMANCE, IntPtr.Zero, 0, buf, (uint)buf.Length, out n, IntPtr.Zero) && n >= 64)
          disk = BitConverter.ToInt64(buf, 32) + " " + BitConverter.ToInt64(buf, 56);
      }
    }
    Console.Out.Write(m.Total + " " + m.Avail + " " + idle + " " + kernel + " " + user + " " + disk + "\n");
  }
}
