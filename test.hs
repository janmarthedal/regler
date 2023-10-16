import Parsing ( Parser, (<|>), natural, symbol, parse )

expr :: Parser Int
expr = do t <- term
          do symbol "+"
             e <- expr
             return (t + e)
           <|> return t

term :: Parser Int
term = do f <- factor
          do symbol "*"
             t <- term
             return (f * t)
           <|> return f

factor :: Parser Int
factor = do symbol "("
            e <- expr
            symbol ")"
            return e
          <|> natural

eval :: String -> Int
eval xs = case parse expr xs of
            Just (n, [])  -> n
            Just (_, out) -> error ("Unused input " ++ out)
            Nothing       -> error "Invalid input"
