# Rust AI Playground

A collection of Rust experiments and tools for working with AI and APIs.

## Features

- Hotel search assistant using the Tripadvisor API
- Example models and API response handling
- Error handling and robust deserialization
- Written in idiomatic Rust

## Projects - AI Agentic Projects in Rust

This repository contains several Rust-based projects demonstrating advanced AI agent workflows, retrieval-augmented generation (RAG), and conversational memory using the Rig AI framework and other modern tools.

---

### agentic-rag-rust-qdrant

Builds an agentic RAG workflow in Rust. The agent can take a CSV file, parse and embed it into Qdrant, and retrieve relevant embeddings to answer user questions about the CSV contents.  
Agentic RAG (Agentic Retrieval Augmented Generation) combines AI agents with RAG, allowing each agent to access vector database embeddings for contextually relevant data—resulting in more accurate answers tailored to specific use cases.

---

### ai-memory-cli

A CLI tool using the Rig AI framework and MongoDB for retrieval-augmented generation (RAG).  
This tool stores summarized conversations in a database and retrieves them as needed, enabling the AI to maintain both short-term and long-term conversational memory for more contextually aware responses.

---

### arxiv-rig-rust

A tutorial project using the Rig AI framework to create an AI agent that suggests research papers based on a given subject.  
Rig is a Rust framework for building agentic pipelines, integrating RAG, and exposing APIs for custom tools. The framework is actively maintained and evolving, with community events like the ARC Handshake showcasing new AI agents.

---

### flight_search_assistant

A step-by-step guide and implementation for building a Flight Search AI Assistant in Rust using the Rig library.  
By following this project, you'll learn Rust fundamentals, how to set up AI agents with custom tools, and how Rig simplifies building an agent that finds the cheapest flights between two airports.

---

### semantic-router

Demonstrates building an efficient semantic router using Qdrant, Rig, and Rust.  
This project shows how to combine Qdrant’s vector search, the Rig LLM framework, and Rust’s performance to create a system that empowers agents to make precise, context-aware decisions—protecting against prompt injection and improving decision-making in conversational AI.

---

### hotel_search_assistant

An AI assistant that helps users find hotels using Rust and the Rig framework.  
The agent provides hotel details such as name, rating, price, features, and location, and can answer user questions about hotels in a conversational manner.

---

## Getting Started

1. **Clone the repository:**
   ```sh
   git clone git@github.com:jeremycod/rust-ai-playground.git
   cd rust-ai-playground