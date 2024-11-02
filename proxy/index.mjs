import express from "express"
import postgres from "postgres"

const app = express()
const port = 9876

app.use(express.json())

app.post("/", async (req, res) => {
  const { url, query } = req.body
  const sql = postgres(url)
  const results = await sql.unsafe(query)
  res.json({ results, columns: results.columns })
})

app.listen(port, () => {
  console.log(`Listening on port ${port}`)
})