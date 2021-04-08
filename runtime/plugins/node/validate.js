#!/usr/bin/node
const fs = require('fs');
const esprima = require('esprima');

const file = process.argv[2];
const code = fs.readFileSync(file).toString();

try {
    const res = esprima.parseScript(code, {tolerant: true});
    if (res.errors.length !== 0) {
        for (const error of res.errors) {
            displayError(code, error);
        }
        process.exit(1);
    }
} catch (error) {
    displayError(code, error);
}
process.exit(0);

function displayError(code, error) {
    const split = code.split('\n');
    console.error(split[error.lineNumber - 1]);
    for (let i = 1; i < error.column; i++) {
        process.stderr.write(' ');
    }
    console.error('^');
    console.error(`${error.toString()}
    at (${file}:${error.lineNumber}:${error.column})`);
    console.error('--------------------------------------------------------------------------');
}
