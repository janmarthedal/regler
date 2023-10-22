{-# OPTIONS_GHC -Wno-unrecognised-pragmas #-}
{-# HLINT ignore "Use <$>" #-}

import Parsing ( Parser, (<|>), natural, symbol, parse )

data Expr = Add Expr Expr | Sub Expr Expr | Mul Expr Expr | Div Expr Expr | Neg Expr | Nat Int
            deriving Show

expr :: Parser Expr
expr = do t <- term
          expr' t

expr' :: Expr -> Parser Expr
expr' t1 = do symbol "+"
              t2 <- term
              expr' (Add t1 t2)
            <|> do symbol "-"
                   t2 <- term
                   expr' (Sub t1 t2)
                 <|> return t1

term :: Parser Expr
term = do f <- unary
          term' f

term' :: Expr -> Parser Expr
term' f1 = do symbol "*"
              f2 <- unary
              term' (Mul f1 f2)
            <|> do symbol "/"
                   f2 <- unary
                   term' (Div f1 f2)
                 <|> return f1

unary :: Parser Expr
unary = do symbol "-"
           f <- unary
           return (Neg f)
         <|> factor

factor :: Parser Expr
factor = do symbol "("
            e <- expr
            symbol ")"
            return e
          <|> do n <- natural
                 return (Nat n)

build :: String -> Expr
build xs = case parse expr xs of
             Just (n, [])  -> n
             Just (_, out) -> error ("Unused input " ++ out)
             Nothing       -> error "Invalid input"

showExpr :: Expr -> String
showExpr t = showExprEq t 0

showExprEq :: Expr -> Int -> String
showExprEq (Add e1 e2) n = showAssocEq "+" e1 e2 n 1
showExprEq (Sub e1 e2) n = showAssocLeftEq "-" e1 e2 n 1
showExprEq (Mul e1 e2) n = showAssocEq "*" e1 e2 n 2
showExprEq (Div e1 e2) n = showAssocLeftEq "/" e1 e2 n 2
showExprEq (Neg e) n = parenCheckEq n 3 ("-" ++ showExprEq e 3)
showExprEq (Nat n) _ = show n

showAssocEq :: String -> Expr -> Expr -> Int -> Int -> String
showAssocEq op e1 e2 n m = parenCheckEq n m (showExprEq e1 m ++ op ++ showExprEq e2 m)

showAssocLeftEq :: String -> Expr -> Expr -> Int -> Int -> String
showAssocLeftEq op e1 e2 n m = parenCheckEq n m (showExprEq e1 m ++ op ++ showExprLt e2 m)

parenCheckEq :: Int -> Int -> String -> String
parenCheckEq n m xs = if n <= m then xs else "(" ++ xs ++ ")"

showExprLt :: Expr -> Int -> String
showExprLt (Add e1 e2) n = showAssocLt "+" e1 e2 n 1
showExprLt (Sub e1 e2) n = showAssocLeftLt "-" e1 e2 n 1
showExprLt (Mul e1 e2) n = showAssocLt "*" e1 e2 n 2
showExprLt (Div e1 e2) n = showAssocLeftLt "/" e1 e2 n 2
showExprLt (Neg e) n = parenCheckLt n 3 ("-" ++ showExprEq e 3)
showExprLt (Nat n) _ = show n

showAssocLt :: String -> Expr -> Expr -> Int -> Int -> String
showAssocLt op e1 e2 n m = parenCheckLt n m (showExprEq e1 m ++ op ++ showExprEq e2 m)

showAssocLeftLt :: String -> Expr -> Expr -> Int -> Int -> String
showAssocLeftLt op e1 e2 n m = parenCheckLt n m (showExprEq e1 m ++ op ++ showExprLt e2 m)

parenCheckLt :: Int -> Int -> String -> String
parenCheckLt n m xs = if n < m then xs else "(" ++ xs ++ ")"
