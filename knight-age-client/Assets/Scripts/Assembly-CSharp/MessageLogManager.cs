using UnityEngine;

public class MessageLogManager : MonoBehaviour
{
	private bool showWindow = false;
	private Vector2 scrollPosition = Vector2.zero;
	private string logContent = "";
	private string logDirectory = "";

	void Start()
	{
		logDirectory = MessageLogger.Instance.GetLogDirectory();
		LoadLogContent();
	}

	void OnGUI()
	{
		if (!showWindow) return;

		GUILayout.BeginArea(new Rect(10, 10, 500, 600));
		GUILayout.BeginVertical("box");

		GUILayout.Label("Message Log Manager", GUI.skin.box);
		
		GUILayout.BeginHorizontal();
		if (GUILayout.Button("Refresh Log", GUILayout.Width(120)))
		{
			LoadLogContent();
		}
		if (GUILayout.Button("Clear Current Log", GUILayout.Width(150)))
		{
			MessageLogger.Instance.ClearCurrentLog();
			LoadLogContent();
		}
		if (GUILayout.Button("Clear All Logs", GUILayout.Width(150)))
		{
			MessageLogger.Instance.ClearAllLogs();
			LoadLogContent();
		}
		if (GUILayout.Button("Open Log Folder", GUILayout.Width(150)))
		{
			string folderPath = logDirectory.Replace("\\", "/");
			if (!folderPath.StartsWith("file://"))
			{
				folderPath = "file:///" + folderPath;
			}
			Application.OpenURL(folderPath);
		}
		if (GUILayout.Button("Close", GUILayout.Width(80)))
		{
			showWindow = false;
		}
		GUILayout.EndHorizontal();

		GUILayout.Space(10);
		GUILayout.Label($"Log Directory: {logDirectory}");
		GUILayout.Label($"Current Log File: {MessageLogger.Instance.GetLogFilePath()}");

		GUILayout.Space(10);
		bool enabled = MessageLogger.Instance.IsEnabled();
		bool newEnabled = GUILayout.Toggle(enabled, "Enable Logging");
		if (newEnabled != enabled)
		{
			MessageLogger.Instance.SetEnabled(newEnabled);
		}

		GUILayout.Space(10);
		GUILayout.Label("Log Content (Last 1000 lines):");
		scrollPosition = GUILayout.BeginScrollView(scrollPosition, GUILayout.Height(400));
		GUILayout.TextArea(logContent, GUILayout.ExpandHeight(true));
		GUILayout.EndScrollView();

		GUILayout.EndVertical();
		GUILayout.EndArea();
	}

	private void LoadLogContent()
	{
		try
		{
			string logFile = MessageLogger.Instance.GetLogFilePath();
			if (System.IO.File.Exists(logFile))
			{
				// Read file with FileShare.ReadWrite to allow reading while it's being written
				using (System.IO.FileStream fileStream = new System.IO.FileStream(
					logFile, 
					System.IO.FileMode.Open, 
					System.IO.FileAccess.Read, 
					System.IO.FileShare.ReadWrite))
				{
					using (System.IO.StreamReader reader = new System.IO.StreamReader(fileStream))
					{
						System.Collections.Generic.List<string> lines = new System.Collections.Generic.List<string>();
						string line;
						while ((line = reader.ReadLine()) != null)
						{
							lines.Add(line);
						}
						
						int startLine = Mathf.Max(0, lines.Count - 1000);
						int lineCount = lines.Count - startLine;
						logContent = string.Join("\n", lines.GetRange(startLine, lineCount).ToArray());
					}
				}
			}
			else
			{
				logContent = "Log file not found. Logging may not have started yet.";
			}
		}
		catch (System.Exception ex)
		{
			logContent = "Error loading log: " + ex.Message;
		}
	}

	public static void ShowWindow()
	{
		MessageLogManager manager = FindObjectOfType<MessageLogManager>();
		if (manager == null)
		{
			GameObject go = new GameObject("MessageLogManager");
			manager = go.AddComponent<MessageLogManager>();
		}
		manager.showWindow = true;
		manager.LoadLogContent();
	}

	public static void ToggleWindow()
	{
		MessageLogManager manager = FindObjectOfType<MessageLogManager>();
		if (manager == null)
		{
			GameObject go = new GameObject("MessageLogManager");
			manager = go.AddComponent<MessageLogManager>();
		}
		manager.showWindow = !manager.showWindow;
		if (manager.showWindow)
		{
			manager.LoadLogContent();
		}
	}
}

