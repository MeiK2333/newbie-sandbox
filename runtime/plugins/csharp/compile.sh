#!/bin/bash
cp -r /plugins/csharp/Main/* ./
mv $1 Program.cs
/usr/bin/dotnet build -c release
