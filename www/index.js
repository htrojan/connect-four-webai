import * as wasm from "../pkg"
import {FieldType, GameBoard} from "../pkg";

const GRID_COLOR = "black";

const canvas = document.getElementById("connect-four-canvas");
const ctx = canvas.getContext("2d");

// Width and height of Board
const width = wasm.GameBoard.get_x();
const height = wasm.GameBoard.get_y();

// Width and height of Cells
const cellSize = 60;

const borderWidth = 3;
const offset = borderWidth/2;

let GAME_STATE = FieldType.Player;

canvas.width = (cellSize + borderWidth) * width + borderWidth
canvas.height = (cellSize + borderWidth) * height + borderWidth

let board = wasm.GameBoard.empty();
let winner = undefined;
// board = board.new_with_move(0, FieldType.Computer)
// board = board.new_with_move(1, FieldType.Player)

const renderLoop = () => {
    drawGrid();

    requestAnimationFrame(renderLoop);
}

const checkWin = function() {
    winner = board.evaluate().player_win();

    if (winner === FieldType.Player) {
        console.log("Player won!")

    } else if (winner === FieldType.Computer) {
        console.log("Computer won!")
    } else {
        return false;
    }
    return true;
}

/**
 * @param {number} row
 */
const makeMove = function(row) {
    if (winner !== undefined) {
        return;
    }
    // New move is being calculated
    if (GAME_STATE === FieldType.Computer) {
        // This shouldn't happen!
        return;
    }
    let b = board.new_with_move(row, GAME_STATE);
    if (b === undefined) {
        return;
    }
    board.free();
    board = b;

    if (checkWin()) {
        return;
    }

    GAME_STATE = FieldType.Computer;
    let move = wasm.ABSolver.solve(board, 8, FieldType.Computer);
    console.log("Score = ", move.score)
    let b_new = board.new_with_move(move.move_row, FieldType.Computer);
    board.free()
    board = b_new;
    GAME_STATE = FieldType.Player;
    if (checkWin()) {
        return;
    }
}

const drawGrid = () => {
    ctx.beginPath();
    ctx.strokeStyle = GRID_COLOR;

    // Vertical lines
    for (let i = 0; i <= width+1; i++) {
        ctx.moveTo(i * (cellSize + borderWidth) + offset , offset);
        ctx.lineTo(i * (cellSize + borderWidth) + offset, (cellSize + borderWidth) * height + offset );
    }
    // Horizontal lines
    for (let i = 0; i <= height+1; i++) {
        ctx.moveTo(offset, i * (cellSize + borderWidth) + offset);
        ctx.lineTo((cellSize + borderWidth) * width + offset, i * (cellSize + borderWidth) + offset );
    }
    ctx.stroke();
}

const drawBoard = () => {
    ctx.strokeStyle = "red"
    for (let x = 0; x < width; x++) {
        for (let y = 0; y < height; y++) {
            // Rotate board
            let field = board.at(x, height - y -1);
            if (field !== undefined) {
                ctx.beginPath();
                if (field === FieldType.Computer) {
                    ctx.fillStyle = "red";
                }
                if (field === FieldType.Player) {
                    ctx.fillStyle = "blue";
                }
                ctx.arc(x * (cellSize + borderWidth) + borderWidth + cellSize/2,
                    y * (cellSize + borderWidth) + borderWidth + cellSize/2, cellSize/2,
                    0, 2*Math.PI)
                ctx.fill()
            }
        }
    }
}

canvas.addEventListener("click", event => {
    const boundingRect = canvas.getBoundingClientRect();
    // console.log("Bounding height: ", boundingRect)
    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const row = Math.min(Math.floor(canvasLeft / (cellSize + borderWidth)), width-1);
    // console.log("Row: ", row)
    makeMove(row);
    drawBoard();
})

// makeMove(0);
// makeMove(1);
drawGrid();
drawBoard();