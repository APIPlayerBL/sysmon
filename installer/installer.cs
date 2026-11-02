using System;
using System.Diagnostics;
using System.IO;
using System.Reflection;

class Program
{
    static void Main()
    {
        try
        {
            string destDir = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
                "Programs", "AIDA67");
            Directory.CreateDirectory(destDir);

            string destExe = Path.Combine(destDir, "aida67.exe");
            Assembly asm = Assembly.GetExecutingAssembly();
            using (Stream rs = asm.GetManifestResourceStream("aida67.exe"))
            {
                if (rs == null)
                    throw new Exception("Embedded application resource not found.");
                using (FileStream fs = new FileStream(destExe, FileMode.Create, FileAccess.Write, FileShare.None))
                {
                    rs.CopyTo(fs);
                }
            }

            string shortcutPath = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.Desktop),
                "AIDA67.lnk");
            try
            {
                Type shellType = Type.GetTypeFromProgID("WScript.Shell");
                object shell = Activator.CreateInstance(shellType);
                object shortcut = shellType.InvokeMember("CreateShortcut",
                    System.Reflection.BindingFlags.InvokeMethod, null, shell, new object[] { shortcutPath });
                Type shortcutType = shortcut.GetType();
                shortcutType.InvokeMember("TargetPath",
                    System.Reflection.BindingFlags.SetProperty, null, shortcut, new object[] { destExe });
                shortcutType.InvokeMember("WorkingDirectory",
                    System.Reflection.BindingFlags.SetProperty, null, shortcut, new object[] { destDir });
                shortcutType.InvokeMember("IconLocation",
                    System.Reflection.BindingFlags.SetProperty, null, shortcut, new object[] { destExe + ",0" });
                shortcutType.InvokeMember("Description",
                    System.Reflection.BindingFlags.SetProperty, null, shortcut, new object[] { "AIDA67 - System Information" });
                shortcutType.InvokeMember("Save",
                    System.Reflection.BindingFlags.InvokeMethod, null, shortcut, null);
            }
            catch
            {
                // Shortcut creation is optional; extraction/installation still succeeds.
            }

            Process.Start(destExe);
        }
        catch (Exception ex)
        {
            System.Windows.Forms.MessageBox.Show(
                "Installation failed:\n" + ex.Message,
                "AIDA67", System.Windows.Forms.MessageBoxButtons.OK,
                System.Windows.Forms.MessageBoxIcon.Error);
        }
    }
}