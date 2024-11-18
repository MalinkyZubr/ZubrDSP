* add taps, that allow inspection of data at any given thread, like for GUI integration
* make the stuff Arc and Mutex capable
  * remember Mutex automatically makes Sync, just need to implement Send. Anything with Send should be in a Mutex!