{-# OPTIONS_GHC -Wno-unrecognised-pragmas #-}
{-# HLINT ignore "Use <$>" #-}

import Parsing ( Parser, (<|>), natural, symbol, parse )

data Tree = Sum Tree Tree | Sub Tree Tree | Prod Tree Tree | Div Tree Tree | Nat Int
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
term = do f <- factor
          term' f

term' :: Tree -> Parser Tree
term' f1 = do symbol "*"
              f2 <- factor
              term' (Prod f1 f2)
            <|> do symbol "/"
                   f2 <- factor
                   term' (Div f1 f2)
                 <|> return f1

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
