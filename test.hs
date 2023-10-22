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
showExpr t = showExpr' t (>=0)

showExpr' :: Expr -> (Int -> Bool) -> String
showExpr' (Add e1 e2) parenCmp = showAssoc' "+" e1 e2 parenCmp 1
showExpr' (Sub e1 e2) parenCmp = showAssocLeft' "-" e1 e2 parenCmp 1
showExpr' (Mul e1 e2) parenCmp = showAssoc' "*" e1 e2 parenCmp 2
showExpr' (Div e1 e2) parenCmp = showAssocLeft' "/" e1 e2 parenCmp 2
showExpr' (Neg e) parenCmp = parenCheck (parenCmp 3) ("-" ++ showExpr' e (>=3))
showExpr' (Nat n) _ = show n

showAssoc' :: String -> Expr -> Expr -> (Int -> Bool) -> Int -> String
showAssoc' op e1 e2 parenCmp m = parenCheck (parenCmp m) (showExpr' e1 (>=m) ++ op ++ showExpr' e2 (>=m))

showAssocLeft' :: String -> Expr -> Expr -> (Int -> Bool) -> Int -> String
showAssocLeft' op e1 e2 parenCmp m = parenCheck (parenCmp m) (showExpr' e1 (>=m) ++ op ++ showExpr' e2 (>m))

parenCheck :: Bool -> String -> String
parenCheck noParens ex = if noParens then ex else "(" ++ ex ++ ")"
