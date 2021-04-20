#!/bin/bash
cp -r /plugins/csharp/Main/* ./
mv $1 Program.cs
dotnet build -c release
