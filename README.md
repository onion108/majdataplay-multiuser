# (a naïve approach of) MajdataPlay MultiUser Launcher

This launcher need to be placed in the same directory as `MajdataPlay.exe`. Launching through this application provides:
- Isolate user's settings and score data.
- Add user if you want.

All user data are stored in `UserProfile/` directory, edit subdirectories' names or delete them to make changes.
Notice that first launch will wipe out all existing global data. You can preserve them by manually create an `UserData` directory, create an subdirectory with desired username, and put `settings.json` and `Majdata.db.db.db.db..... (a lot of db)` into the subdirectory.

The game [MajdataPlay](https://github.com/LingFeng-bbben/MajdataPlay) isn't included in this directory.

