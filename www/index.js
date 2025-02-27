import * as wasm from "../solver/pkg"
import {BitBoard, FieldType} from "../solver/pkg";



const GRID_COLOR = "black";

const canvas = document.getElementById("connect-four-canvas");
const ctx = canvas.getContext("2d");

// Width and height of Board
const width = 7;
const height = 6;

// Width and height of Cells
const cellSize = 60;

const borderWidth = 3;
const offset = borderWidth/2;

let GAME_STATE = FieldType.Opponent;

canvas.width = (cellSize + borderWidth) * width + borderWidth
canvas.height = (cellSize + borderWidth) * height + borderWidth

let board = BitBoard.empty();
let winner = undefined;
let last_guess = 3;
// board = board.new_with_move(0, FieldType.Computer)
// board = board.new_with_move(1, FieldType.Player)

const renderLoop = () => {
    drawGrid();

    requestAnimationFrame(renderLoop);
}

const checkWin = function() {
    if (board.has_won()) {
        console.log("Computer won!")
        winner = FieldType.Player;
        return true;
    } else if (board.has_lost()) {
        console.log("Player won!")
        winner = FieldType.Opponent;
        return true;
    } else {
        return false;
    }
}

/**
 * The player is the computer.
 * The opponent is the physical player playing the website.
 * @param {number} row
 */
const makeMove = function(row) {
    // Game already over
    if (winner !== undefined) {
        return;
    }

    // New move is being calculated
    if (GAME_STATE === FieldType.Player) {
        // This shouldn't happen!
        return;
    }

    let b = board.play_column(row, FieldType.Opponent);
    if (b === undefined) {
        return
    }

    board.free()
    board = b

    console.log("Player made move")
    if (checkWin()) {
        return;
    }

    GAME_STATE = FieldType.Player;

    let move = 0;

    console.log("Number of stones: ", board.number_of_stones())
    let t1 = new Date().getTime();
    if (board.number_of_stones() >= 15) {
        console.log("[Endgame] Solving Complete board")
        move = wasm.solve(board, 42, wasm.SolverType.Weak);
    }
    else {
        console.log("[Earlygame] Solving with heuristic score and depth: 15")
        move = wasm.solve(board, 17, wasm.SolverType.Strong);
    }
    let t2 = new Date().getTime();
    console.log("Time: ", (t2-t1),"ms, Searched ", move.nodes_searched.toLocaleString(), " Nodes")
    let perf = "Nan"
    if ((t2-t1) > 0) {
        perf = (move.nodes_searched/BigInt(t2-t1)).toLocaleString()
    }
    console.log("Performance: ", perf, "kN/s")


    if (move.end_in === 0) {
        console.log("Score = ", move.score)
    } else if (move.score > 0){
        console.log("Computer wins in ", move.end_in)
    } else {
        console.log("Player wins in ", move.end_in)
    }
    // let b_new = board.new_with_move(move.move_row, FieldType.Opponent);
    board.set_at(move.mov, FieldType.Player)

    GAME_STATE = FieldType.Opponent;

    checkWin();
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
    console.log("Drawing board")
    ctx.strokeStyle = "red"
    for (let x = 0; x < width; x++) {
        for (let y = 0; y < height; y++) {
            // Rotate board
            let field = board.get_at(x, height - y -1);
            if (field !== undefined) {
                ctx.beginPath();
                if (field === FieldType.Opponent) {
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