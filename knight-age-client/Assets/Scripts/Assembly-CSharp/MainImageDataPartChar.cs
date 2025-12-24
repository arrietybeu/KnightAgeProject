using System;

public class MainImageDataPartChar
{
	public mImage img;

	public int count = -1;

	public sbyte type;

	public short id;

	public short version;

	public long timeImageNull;

	public sbyte[] isData;

	public sbyte[] isDataImg;

	public MainImageDataPartChar()
	{
	}

	public MainImageDataPartChar(sbyte[] issImg, sbyte[] iss, sbyte type, short id, short version)
	{
		isDataImg = issImg;
		count = 0;
		isData = iss;
		this.type = type;
		this.id = id;
		this.version = version;
	}

	public MainImageDataPartChar(mImage img, sbyte[] iss)
	{
		this.img = img;
		count = 0;
		isData = iss;
	}

	public DataOutputStream getSaveData()
	{
		DataOutputStream dataOutputStream = new DataOutputStream();
		try
		{
			dataOutputStream.writeShort(version);
			dataOutputStream.writeInt(isDataImg.Length);
			dataOutputStream.write(isDataImg);
			dataOutputStream.writeShort(isData.Length);
			dataOutputStream.write(isData);
		}
		catch (Exception)
		{
		}
		return dataOutputStream;
	}
}
