return 42; // should be inside a function
function f() {
    'use strict';
    var x = 042;
    with (z) { }
}
