use std::io;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, SocketAddr};
fn main() {
    /*
    TODO
    reformat checking code to have row_diff for everything and check if same pos using it
     */
    let chessboard: Vec<Vec<Tile>> = vec![vec![Tile::Nothing; 8]; 8];
    let mut chessboard = init_board(chessboard);
    let mut round_num: u16 = 0;
    let mut piece_col = 0;
    let mut piece_row = 0;
    let mut loc_col = 0;
    let mut loc_row = 0;
    println!("Welcome to chess, made in Rust!");
    let mut multiplayer = check_multiplayer();
    loop {
        //check if its white or blacks turn
        round_num += 1;
        match multiplayer {
            MultiplayerResult::Hosting(ref mut stream) => {
                {
                    println!("Waiting for opponent to play");
                    let mut buffer = [0; 8];
                    match stream.read(&mut buffer) {
                        Ok(_) => {
                            let received_data = String::from_utf8_lossy(&buffer[..]);
                            let moves = received_data.split(",").collect::<Vec<&str>>();
                            let (location_col, location_row) = convert_pos(&moves[1].to_string());
                            let (player_col, player_row) = convert_pos(&moves[0].to_string());
                            chessboard[location_row as usize][location_col as usize] = chessboard[player_row as usize][player_col as usize].clone();
                            chessboard[player_row as usize][player_col as usize] = Tile::Nothing;
                            round_num += 1;
                        }
                        Err(error) => {
                            println!("Error reading from socket: {}", error);
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
        let mut is_valid = ValidateResult::Failure;
        print_board(&chessboard);
        println!("");
        while let ValidateResult::Failure = is_valid{
            println!("Input the location of the piece you want to move: (ex: h4)");
            (piece_col, piece_row) = get_pos();
            match &chessboard[piece_row as usize][piece_col as usize] {
                Tile::Something(piece) => match piece {
                    Piece {
                        piece_type: _,
                        color: Color::White,
                    } => if round_num % 2 != 1 {println!("You used a white piece, but it is not white's turn."); continue;},
                    _ => if round_num % 2 == 1 {println!("You used a black piece, but it is not black's turn."); continue;},
                },
                Tile::Nothing => {}
            }
            println!("Input the location of where you want to move it to: (ex: f8)");
            (loc_col, loc_row) = get_pos();
            is_valid = check_if_valid(
                &chessboard,
                &chessboard[piece_row as usize][piece_col as usize],
                &chessboard[loc_row as usize][loc_col as usize],
                piece_row as usize,
                piece_col as usize,
                loc_row as usize,
                loc_col as usize,
            );
        }
        chessboard[loc_row as usize][loc_col as usize] = chessboard[piece_row as usize][piece_col as usize].clone();
        chessboard[piece_row as usize][piece_col as usize] = Tile::Nothing;
        match multiplayer {
            MultiplayerResult::Joining(ref mut stream) => {
                {
                    stream.write_all(format!("{}{},{}{}", piece_col,piece_row, loc_col,loc_row).as_bytes()).expect("Failed to send data");
                    round_num += 1;
                    print_board(&chessboard);
                    println!("Waiting for opponent to play");
                    let mut buffer = [0; 8];
                    let bytes_read = stream.read(&mut buffer).expect("Failed to receive data from server");
                    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                    let moves = response.split(",").collect::<Vec<&str>>();
                    let (location_col, location_row) = convert_pos(&moves[1].to_string());
                    let (player_col, player_row) = convert_pos(&moves[0].to_string());
                    chessboard[location_row as usize][location_col as usize] = chessboard[player_row as usize][player_col as usize].clone();
                    chessboard[player_row as usize][player_col as usize] = Tile::Nothing;
                }
            }
            MultiplayerResult::Hosting(ref mut stream) => {
                {
                    stream.write_all(format!("{}{},{}{}", piece_col,piece_row, loc_col,loc_row).as_bytes()).expect("Failed to send data");
                }
            }
            MultiplayerResult::Local => {}
        }
    }
}

fn check_multiplayer() -> MultiplayerResult {
    println!("Would you like to play online or locally? (type ON or LO):");
    let mut play = String::new();
    io::stdin()
        .read_line(&mut play)
        .expect("Failed to read line");
    if play.contains("ON"){
        println!("Are you hosting the server or joining a game? (H or J):");
        io::stdin()
        .read_line(&mut play)
        .expect("Failed to read line");
        if play.contains("H"){
            return MultiplayerResult::Hosting(run_server());
        }
        else{
            return MultiplayerResult::Joining(join_server());
        }
    }
    else{
        return MultiplayerResult::Local;
    }
}
fn join_server() -> TcpStream{
    println!("What server are you joining? (ex: 127.0.0.1:8080)");
    let mut server = String::new();
    io::stdin()
        .read_line(&mut server)
        .expect("Failed to read line");
    let server = server.trim();
    let socket_addr: SocketAddr = server.parse()
        .expect("Failed to parse server address");
    let mut stream = TcpStream::connect(socket_addr).expect("Failed to connect to server");
    println!("Connected to server");
    stream
}
fn run_server() -> TcpStream{
    println!("What is the ip and address of the server you are running (your ip and a port you've forwarded)?");
    let mut server = String::new();
    io::stdin()
        .read_line(&mut server)
        .expect("Failed to read line");
    // Start the server and listen for incoming connections
    println!("Your server: {}", server);
    // Trim any leading or trailing whitespace from the server string
    let server = server.trim();
    let socket_addr: SocketAddr = server.parse()
        .expect("Failed to parse server address");

    let listener = TcpListener::bind(socket_addr).expect("Failed to bind address");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread for each client connection
                println!("Opponent joined.");
                    return stream;
            }
            Err(error) => {
                println!("Error establishing connection: {}", error);
            }
        }
    }
    return TcpStream::connect("127.0.0.1:8080").expect("Failed to connect to server");
}
enum MultiplayerResult{
    Local,
    Hosting(TcpStream),
    Joining(TcpStream),
}
enum ValidateResult {
    Success,
    Failure,
    SuccessfulPawnPromotion,
}
#[derive(PartialEq)]
enum TileColor {
    White,
    Black,
    None,
}
fn convert_pos(pos: &String) -> (u8,u8){
    let ch1 = pos.chars().nth(0).unwrap().to_digit(10).unwrap() as u8;
    let ch2 = pos.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
    return (ch1, ch2);
}
fn check_if_valid(
    board: &Vec<Vec<Tile>>,
    loc1: &Tile,
    loc2: &Tile,
    piece_row: usize,
    piece_col: usize,
    loc_row: usize,
    loc_col: usize
) -> ValidateResult {
    /*
       logic:
           for everything check if the location is the same, because then it didnt move
       check if lands on opposite color
       pawn: if loc not empty then check if diagonal, else check if forward one or if round = 1 and forward 2
       bishop: subtract two locs and get absolute value, then check if x = y for that, if so it is diagonal. Then check each square leading up to see if there is a piece there.
       rook: check if two rows are same or two cols are same but not both, then check each square leading up to it
       knight: check all combinations of row1 +- 2 and col1 +- 1
       queen: do both rook and bishop
       king: check if distance is 1 and then eventually check if moving puts him in check
       check for pawn promotions
       add en passant?
       CHANGE ROUNDNUM to instead have the pieces have a numMoved and check if pawns is zero
       check if piece it lands on is same color
    */
    if piece_row == loc_row && piece_col == loc_col {
        println!("You input the same location twice!");
        return ValidateResult::Failure;
    }
    let mut loc1_color = TileColor::None;
    let mut loc2_color = TileColor::None;
    match loc2 {
        Tile::Something(piece) => match piece {
            Piece {
                piece_type: _,
                color: Color::White,
            } => loc2_color = TileColor::White,
            _ => {loc2_color = TileColor::Black}
        },
        Tile::Nothing => {}
    }
    match loc1 {
        Tile::Something(piece) => match piece {
            Piece {
                piece_type: _,
                color: Color::White,
            } => loc1_color = TileColor::White,
            _ => {loc1_color = TileColor::Black}
        },
        Tile::Nothing => {}
    }
    if  loc1_color.eq(&loc2_color)
    {
        if !matches!(loc1_color, TileColor::None){
        println!("You cannot capture your own piece");
        return ValidateResult::Failure;
        }
        else{
            println!("You cannot access two empty tiles!");
            return ValidateResult::Failure;
        }
    }
    match loc1 {
        Tile::Something(piece) => match piece {
            Piece {
                color: pawn_color,
                piece_type: PieceType::Pawn(round_num),
            } => {
                if let Color::White = pawn_color{
                    if let Tile::Nothing = loc2 {
                        if let Tile::Something(_) = board[piece_row - 1][piece_col] {
                            println!(
                                "You attempted to push a pawn, but there was a tile in front of it"
                            );
                            return ValidateResult::Failure;
                        }
                        if loc_col != piece_col {
                            println!("You moved a pawn sideways/diagonally when there was nothing to capture.");
                            return ValidateResult::Failure;
                        }
                        if loc_row == piece_row - 2 {
                            if *round_num == 0 {
                                if let Tile::Something(_) = board[piece_row - 2][piece_col] {
                                    println!("You attempted to push a pawn, but there was a tile where it wanted to go");
                                    return ValidateResult::Failure;
                                } else {
                                    return ValidateResult::Success;
                                }
                            } else {
                                println!("You cannot move that far forward");
                            }
                        } else {
                            if loc_row == piece_row - 1 {
                                if loc_row == 0 || loc_row == 7 {
                                    println!("Pawn promotion! You can promote the pawn to a queen.");
                                    return ValidateResult::SuccessfulPawnPromotion;
                                }
                                return ValidateResult::Success;
                            } else {
                                println!("You cannot move that far forward");
                            }
                        }
                    } else {
                        if loc_row == piece_row - 1
                            && (loc_col == piece_col + 1 || loc_col == piece_col - 1)
                        {
                            return ValidateResult::Success;
                        } else {
                            println!("You tried to move a pawn too far diagonally or it hit a black piece somewhere");
                            return ValidateResult::Failure;
                        }
                    }
                }
                else{
                    if let Tile::Nothing = loc2 {
                        if let Tile::Something(_) = board[piece_row + 1][piece_col] {
                            println!(
                                "You attempted to push a pawn, but there was a tile in front of it"
                            );
                            return ValidateResult::Failure;
                        }
                        if loc_col != piece_col {
                            println!("You moved a pawn sideways/diagonally when there was nothing to capture.");
                            return ValidateResult::Failure;
                        }
                        if loc_row == piece_row + 2 {
                            if *round_num == 0 {
                                if let Tile::Something(_) = board[piece_row + 2][piece_col] {
                                    println!("You attempted to push a pawn, but there was a tile where it wanted to go");
                                    return ValidateResult::Failure;
                                } else {
                                    return ValidateResult::Success;
                                }
                            } else {
                                println!("You cannot move that far forward");
                            }
                        } else {
                            if loc_row == piece_row + 1 {
                                if loc_row == 0 || loc_row == 7 {
                                    println!("Pawn promotion! You can promote the pawn to a queen.");
                                    return ValidateResult::SuccessfulPawnPromotion;
                                }
                                return ValidateResult::Success;
                            } else {
                                println!("You cannot move that far forward");
                            }
                        }
                    } else {
                        if loc_row == piece_row + 1
                            && (loc_col == piece_col + 1 || loc_col == piece_col - 1)
                        {
                            return ValidateResult::Success;
                        } else {
                            println!("You tried to move a pawn too far diagonally or it hit a black piece somewhere");
                            return ValidateResult::Failure;
                        }
                    }
                }
            }
            Piece {
                color: _,
                piece_type: PieceType::Bishop,
            } => {
                let row_diff = (loc_row as i32 - piece_row as i32).abs();
                let col_diff = (loc_col as i32 - piece_col as i32).abs();

                if row_diff != col_diff {
                    println!("Bishops can only move diagonally");
                    return ValidateResult::Failure;
                }

                let row_step: isize = if loc_row > piece_row { 1 } else { -1 };
                let col_step: isize = if loc_col > piece_col { 1 } else { -1 };

                let mut current_row = piece_row as isize + row_step;
                let mut current_col = piece_col as isize + col_step;

                while current_row != loc_row as isize {
                    if let Tile::Something(_) = board[current_row as usize][current_col as usize] {
                        println!("There is an obstruction in the path of the bishop");
                        return ValidateResult::Failure;
                    }
                    current_row += row_step;
                    current_col += col_step;
                }

                return ValidateResult::Success;

            }
            Piece {
                color: _,
                piece_type: PieceType::Rook,
            } => {
                let row_diff = (loc_row as i32 - piece_row as i32).abs();
                let col_diff = (loc_col as i32 - piece_col as i32).abs();

                if row_diff != 0 && col_diff != 0 {
                    println!("Rooks can only move horizontally or vertically");
                    return ValidateResult::Failure;
                }

                if row_diff > 0 {
                    let row_step: isize = if loc_row > piece_row { 1 } else { -1 };

                    let mut current_row = piece_row as isize + row_step;
                    while current_row != loc_row as isize {
                        if let Tile::Something(_) = board[current_row as usize][piece_col] {
                            println!("There is an obstruction in the path of the rook");
                            return ValidateResult::Failure;
                        }
                        current_row += row_step;
                    }
                }

                if col_diff > 0 {
                    let col_step = if loc_col > piece_col { 1 } else { -1 as isize };

                    let mut current_col = piece_col as isize + col_step;
                    while current_col != loc_col as isize {
                        if let Tile::Something(_) = board[piece_row][current_col as usize] {
                            println!("There is an obstruction in the path of the rook");
                            return ValidateResult::Failure;
                        }
                        current_col += col_step;
                    }
                }

                return ValidateResult::Success;

            }
            Piece {
                color: _,
                piece_type: PieceType::Knight,
            } => {
                let row_diff = (loc_row as i32 - piece_row as i32).abs();
                let col_diff = (loc_col as i32 - piece_col as i32).abs();

                if (row_diff == 2 && col_diff == 1) || (row_diff == 1 && col_diff == 2) {
                    return ValidateResult::Success;
                }

                println!("Knights can only move in an L-shape");
                return ValidateResult::Failure;
            }
            Piece {
                color: _,
                piece_type: PieceType::Queen,
            } => {
                let row_diff = (loc_row as i32 - piece_row as i32).abs();
                let col_diff = (loc_col as i32 - piece_col as i32).abs();

                if row_diff == 0 || col_diff == 0 || row_diff == col_diff {
                    if row_diff == 0 && col_diff == 0 {
                        println!("Queen cannot stay in the same position");
                        return ValidateResult::Failure;
                    }

                    let row_step: isize = if loc_row > piece_row { 1 } else if loc_row < piece_row { -1 } else { 0 };
                    let col_step: isize = if loc_col > piece_col { 1 } else if loc_col < piece_col { -1 } else { 0 };

                    let mut current_row = piece_row as isize + row_step;
                    let mut current_col = piece_col as isize + col_step;

                    while current_row != loc_row as isize || current_col != loc_col  as isize{
                        if let Tile::Something(_) = board[current_row as usize][current_col as usize] {
                            println!("There is an obstruction in the path of the queen");
                            return ValidateResult::Failure;
                        }
                        current_row += row_step;
                        current_col += col_step;
                    }

                    return ValidateResult::Success;
                }

                println!("Queen can move horizontally, vertically, or diagonally");
                return ValidateResult::Failure;

            }
            Piece {
                color: _,
                piece_type: PieceType::King,
            } => {
                let row_diff = (loc_row as i32 - piece_row as i32).abs();
                let col_diff = (loc_col as i32 - piece_col as i32).abs();

                if row_diff <= 1 && col_diff <= 1 {
                    return ValidateResult::Success;
                }

                println!("King can only move one square in any direction");
                return ValidateResult::Failure;
            }
        },
        _ => println!("You selected an empty tile to move."),
    }
    return ValidateResult::Failure;
}
fn get_pos() -> (u8, u8) {
    loop {
        let mut can_return: u8 = u8::MAX;
        let mut piece = String::new();
        io::stdin()
            .read_line(&mut piece)
            .expect("Failed to read line");
        let ch1 = piece.chars().nth(0).unwrap();
        let ch2 = piece.chars().nth(1).unwrap().to_digit(10);
        let pos_for_ch1;
        if ch1.is_alphabetic(){
        pos_for_ch1 = ((ch1.to_ascii_uppercase() as u8) - b'A') + 1;
        }
        else{
            println!("You did not input a letter a-h for your first char");
            continue;
        }
        match pos_for_ch1 {
            1..=8 => match ch2 {
                Some(n) => can_return = n as u8,
                None => println!("Failed to convert second character to an number"),
            },
            _ => eprintln!("Error: '{}' is not a valid letter a-h", ch1),
        }
        if can_return != u8::MAX {
            return (pos_for_ch1 - 1, 8 - can_return);
        }
    }
}
fn init_board(mut board: Vec<Vec<Tile>>) -> Vec<Vec<Tile>> {
    board[0][0] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Rook,
    });
    board[0][1] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Knight,
    });
    board[0][2] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Bishop,
    });
    board[0][3] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Queen,
    });
    board[0][4] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::King,
    });
    board[0][5] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Bishop,
    });
    board[0][6] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Knight,
    });
    board[0][7] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Rook,
    });
    board[1][0] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Pawn(0),
    });
    board[1][1] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Pawn(0),
    });
    board[1][2] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Pawn(0),
    });
    board[1][3] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Pawn(0),
    });
    board[1][4] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Pawn(0),
    });
    board[1][5] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Pawn(0),
    });
    board[1][6] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Pawn(0),
    });
    board[1][7] = Tile::Something(Piece {
        color: Color::Black,
        piece_type: PieceType::Pawn(0),
    });

    board[7][0] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Rook,
    });
    board[7][1] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Knight,
    });
    board[7][2] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Bishop,
    });
    board[7][3] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Queen,
    });
    board[7][4] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::King,
    });
    board[7][5] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Bishop,
    });
    board[7][6] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Knight,
    });
    board[7][7] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Rook,
    });
    board[6][0] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Pawn(0),
    });
    board[6][1] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Pawn(0),
    });
    board[6][2] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Pawn(0),
    });
    board[6][3] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Pawn(0),
    });
    board[6][4] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Pawn(0),
    });
    board[6][5] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Pawn(0),
    });
    board[6][6] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Pawn(0),
    });
    board[6][7] = Tile::Something(Piece {
        color: Color::White,
        piece_type: PieceType::Pawn(0),
    });
    board
}
fn print_board(board: &Vec<Vec<Tile>>) {
    let mut index = 0;
    println!("");
    for row in board {
        index += 1;
        println!("");
        print!("{} ", (9-index));
        for tile in row {
            match tile {
                Tile::Nothing => print!(" "),
                Tile::Something(piece) => print_piece(&piece),
            }
            print!(" ");
        }
    }
    println!("");
    print!("  a b c d e f g h");
    println!("");
}
fn print_piece(piece: &Piece) {
    match piece {
        Piece {
            color: Color::White,
            piece_type: PieceType::King,
        } => print!("♚"),
        Piece {
            color: Color::White,
            piece_type: PieceType::Pawn(_),
        } => print!("♟"),
        Piece {
            color: Color::White,
            piece_type: PieceType::Queen,
        } => print!("♛"),
        Piece {
            color: Color::White,
            piece_type: PieceType::Knight,
        } => print!("♞"),
        Piece {
            color: Color::White,
            piece_type: PieceType::Rook,
        } => print!("♜"),
        Piece {
            color: Color::White,
            piece_type: PieceType::Bishop,
        } => print!("♝"),
        Piece {
            color: Color::Black,
            piece_type: PieceType::King,
        } => print!("♔"),
        Piece {
            color: Color::Black,
            piece_type: PieceType::Pawn(_),
        } => print!("♙"),
        Piece {
            color: Color::Black,
            piece_type: PieceType::Queen,
        } => print!("♕"),
        Piece {
            color: Color::Black,
            piece_type: PieceType::Knight,
        } => print!("♘"),
        Piece {
            color: Color::Black,
            piece_type: PieceType::Rook,
        } => print!("♖"),
        Piece {
            color: Color::Black,
            piece_type: PieceType::Bishop,
        } => print!("♗")
    }
}
#[derive(Clone)]
enum Color {
    White,
    Black,
}
#[derive(Clone)]
enum PieceType {
    Bishop,
    Rook,
    Knight,
    Queen,
    King,
    Pawn(u16),
}
#[derive(Clone)]
struct Piece {
    color: Color,
    piece_type: PieceType,
}
#[derive(Clone)]
enum Tile {
    Nothing,
    Something(Piece),
}