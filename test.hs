{-# OPTIONS_GHC -Wno-unrecognised-pragmas #-}
{-# HLINT ignore "Use <$>" #-}

import Parsing ( Parser, (<|>), natural, symbol, parse )

data Tree = Sum Tree Tree | Sub Tree Tree | Prod Tree Tree | Div Tree Tree | Neg Tree | Nat Int
            deriving Show

expr :: Parser Tree
expr = do t <- term
          expr' t

expr' :: Tree -> Parser Tree
expr' t1 = do symbol "+"
              t2 <- term
              expr' (Sum t1 t2)
            <|> do symbol "-"
                   t2 <- term
                   expr' (Sub t1 t2)
                 <|> return t1

term :: Parser Tree
term = do f <- unary
          term' f

term' :: Tree -> Parser Tree
term' f1 = do symbol "*"
              f2 <- unary
              term' (Prod f1 f2)
            <|> do symbol "/"
                   f2 <- unary
                   term' (Div f1 f2)
                 <|> return f1

unary :: Parser Tree
unary = do symbol "-"
           f <- unary
           return (Neg f)
         <|> factor

factor :: Parser Tree
factor = do symbol "("
            e <- expr
            symbol ")"
            return e
          <|> do n <- natural
                 return (Nat n)

build :: String -> Tree
build xs = case parse expr xs of
             Just (n, [])  -> n
             Just (_, out) -> error ("Unused input " ++ out)
             Nothing       -> error "Invalid input"

showExpr :: Tree -> String
showExpr t = showExprEq t 0

showExprEq :: Tree -> Int -> String
showExprEq (Sum t1 t2) n = parenCheckEq n 1 (showExprEq t1 1 ++ "+" ++ showExprEq t2 1)
showExprEq (Sub t1 t2) n = parenCheckEq n 1 (showExprEq t1 1 ++ "-" ++ showExprLt t2 1)
showExprEq (Prod t1 t2) n = parenCheckEq n 2 (showExprEq t1 2 ++ "*" ++ showExprEq t2 2)
showExprEq (Div t1 t2) n = parenCheckEq n 2 (showExprEq t1 2 ++ "/" ++ showExprLt t2 2)
showExprEq (Nat n) _ = show n

parenCheckEq :: Int -> Int -> String -> String
parenCheckEq n m xs = if n <= m then xs else "(" ++ xs ++ ")"

showExprLt :: Tree -> Int -> String
showExprLt (Sum t1 t2) n = parenCheckLt n 1 (showExprEq t1 1 ++ "+" ++ showExprEq t2 1)
showExprLt (Sub t1 t2) n = parenCheckLt n 1 (showExprEq t1 1 ++ "-" ++ showExprLt t2 1)
showExprLt (Prod t1 t2) n = parenCheckLt n 2 (showExprEq t1 2 ++ "*" ++ showExprEq t2 2)
showExprLt (Div t1 t2) n = parenCheckLt n 2 (showExprEq t1 2 ++ "/" ++ showExprLt t2 2)
showExprLt (Nat n) _ = show n

parenCheckLt :: Int -> Int -> String -> String
parenCheckLt n m xs = if n < m then xs else "(" ++ xs ++ ")"
