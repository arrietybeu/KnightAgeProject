using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Text;
using UnityEngine;

public class MessageLogger
{
	private static MessageLogger instance;
	private static readonly object lockObject = new object();
	private string logFilePath;
	private bool isEnabled = true;
	private StreamWriter logWriter;
	private int logCount = 0;
	private const int MAX_LOG_ENTRIES_PER_FILE = 10000;
	private const string LOG_DIRECTORY = "MessageLogs";

	public static MessageLogger Instance
	{
		get
		{
			if (instance == null)
			{
				lock (lockObject)
				{
					if (instance == null)
					{
						instance = new MessageLogger();
					}
				}
			}
			return instance;
		}
	}

	private MessageLogger()
	{
		InitializeLogFile();
	}

	private void InitializeLogFile()
	{
		try
		{
			string persistentDataPath = Application.persistentDataPath;
			string logDir = Path.Combine(persistentDataPath, LOG_DIRECTORY);
			if (!Directory.Exists(logDir))
			{
				Directory.CreateDirectory(logDir);
			}

			string timestamp = DateTime.Now.ToString("yyyy-MM-dd_HH-mm-ss");
			logFilePath = Path.Combine(logDir, $"message_log_{timestamp}.txt");
			logWriter = new StreamWriter(logFilePath, true, Encoding.UTF8);
			logWriter.AutoFlush = true;
			
			logWriter.WriteLine("=".PadRight(100, '='));
			logWriter.WriteLine($"Message Log Started: {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
			logWriter.WriteLine($"Log File: {logFilePath}");
			logWriter.WriteLine("=".PadRight(100, '='));
			logWriter.WriteLine();
		}
		catch (Exception ex)
		{
			Debug.LogError($"Failed to initialize message logger: {ex.Message}");
			isEnabled = false;
		}
	}

	public static string GetCommandName(sbyte commandId)
	{
		try
		{
			Type cmdType = typeof(Cmd_Message);
			FieldInfo[] fields = cmdType.GetFields(BindingFlags.Public | BindingFlags.Static | BindingFlags.FlattenHierarchy);
			
			foreach (FieldInfo field in fields)
			{
				if (field.FieldType == typeof(sbyte) && field.IsLiteral && !field.IsInitOnly)
				{
					sbyte value = (sbyte)field.GetValue(null);
					if (value == commandId)
					{
						return field.Name;
					}
				}
			}
		}
		catch (Exception ex)
		{
			Debug.LogError($"Error getting command name: {ex.Message}");
		}
		
		return $"UNKNOWN_CMD_{commandId}";
	}

	public void LogReadMessage(sbyte command, sbyte[] data)
	{
		if (!isEnabled) return;

		lock (lockObject)
		{
			try
			{
				logCount++;
				if (logCount > MAX_LOG_ENTRIES_PER_FILE)
				{
					RotateLogFile();
				}

				string commandName = GetCommandName(command);
				int dataLength = data != null ? data.Length : 0;
				string timestamp = DateTime.Now.ToString("HH:mm:ss.fff");

				logWriter.WriteLine($"[{timestamp}] [READ ] CMD: {command,4} | NAME: {commandName,-35} | SIZE: {dataLength,5} bytes");
				
				// Log hex data preview (first 32 bytes)
				if (data != null && data.Length > 0)
				{
					int previewLength = System.Math.Min(32, data.Length);
					byte[] byteArray = new byte[previewLength];
					for (int i = 0; i < previewLength; i++)
					{
						byteArray[i] = (byte)(data[i] < 0 ? data[i] + 256 : data[i]);
					}
					string hexPreview = BitConverter.ToString(byteArray).Replace("-", " ");
					logWriter.WriteLine($"        └─ Data Preview: {hexPreview}" + (data.Length > 32 ? "..." : ""));
				}
				
				logWriter.Flush();
			}
			catch (Exception ex)
			{
				Debug.LogError($"Error logging read message: {ex.Message}");
			}
		}
	}

	public void LogWriteMessage(sbyte command, sbyte[] data)
	{
		if (!isEnabled) return;

		lock (lockObject)
		{
			try
			{
				logCount++;
				if (logCount > MAX_LOG_ENTRIES_PER_FILE)
				{
					RotateLogFile();
				}

				string commandName = GetCommandName(command);
				int dataLength = data != null ? data.Length : 0;
				string timestamp = DateTime.Now.ToString("HH:mm:ss.fff");

				logWriter.WriteLine($"[{timestamp}] [WRITE] CMD: {command,4} | NAME: {commandName,-35} | SIZE: {dataLength,5} bytes");
				
				// Log hex data preview (first 32 bytes)
				if (data != null && data.Length > 0)
				{
					int previewLength = System.Math.Min(32, data.Length);
					byte[] byteArray = new byte[previewLength];
					for (int i = 0; i < previewLength; i++)
					{
						byteArray[i] = (byte)(data[i] < 0 ? data[i] + 256 : data[i]);
					}
					string hexPreview = BitConverter.ToString(byteArray).Replace("-", " ");
					logWriter.WriteLine($"        └─ Data Preview: {hexPreview}" + (data.Length > 32 ? "..." : ""));
				}
				
				logWriter.Flush();
			}
			catch (Exception ex)
			{
				Debug.LogError($"Error logging write message: {ex.Message}");
			}
		}
	}

	private void RotateLogFile()
	{
		try
		{
			if (logWriter != null)
			{
				logWriter.Close();
				logWriter = null;
			}
			logCount = 0;
			InitializeLogFile();
		}
		catch (Exception ex)
		{
			Debug.LogError($"Error rotating log file: {ex.Message}");
		}
	}

	public void ClearCurrentLog()
	{
		lock (lockObject)
		{
			try
			{
				if (logWriter != null)
				{
					logWriter.Close();
					logWriter = null;
				}
				
				if (File.Exists(logFilePath))
				{
					File.Delete(logFilePath);
				}
				
				logCount = 0;
				InitializeLogFile();
				Debug.Log("Message log cleared!");
			}
			catch (Exception ex)
			{
				Debug.LogError($"Error clearing log: {ex.Message}");
			}
		}
	}

	public void ClearAllLogs()
	{
		lock (lockObject)
		{
			try
			{
				if (logWriter != null)
				{
					logWriter.Close();
					logWriter = null;
				}

				string persistentDataPath = Application.persistentDataPath;
				string logDir = Path.Combine(persistentDataPath, LOG_DIRECTORY);
				
				if (Directory.Exists(logDir))
				{
					Directory.Delete(logDir, true);
					Debug.Log("All message logs cleared!");
				}
				
				logCount = 0;
				InitializeLogFile();
			}
			catch (Exception ex)
			{
				Debug.LogError($"Error clearing all logs: {ex.Message}");
			}
		}
	}

	public string GetLogFilePath()
	{
		return logFilePath;
	}

	public string GetLogDirectory()
	{
		string persistentDataPath = Application.persistentDataPath;
		return Path.Combine(persistentDataPath, LOG_DIRECTORY);
	}

	public void SetEnabled(bool enabled)
	{
		isEnabled = enabled;
	}

	public bool IsEnabled()
	{
		return isEnabled;
	}

	public void Close()
	{
		lock (lockObject)
		{
			if (logWriter != null)
			{
				logWriter.Close();
				logWriter = null;
			}
		}
	}

	~MessageLogger()
	{
		Close();
	}
}

