-- Two People Game Database Schema
-- Run this to initialize your local MySQL database

CREATE DATABASE IF NOT EXISTS two_people;
USE two_people;

-- Questions table - stores all "2 types of people" questions
CREATE TABLE IF NOT EXISTS questions (
    id INT AUTO_INCREMENT PRIMARY KEY,
    question_text VARCHAR(255) NOT NULL,
    option_a_text VARCHAR(100) NOT NULL,
    option_a_emoji VARCHAR(10) NOT NULL,
    option_b_text VARCHAR(100) NOT NULL,
    option_b_emoji VARCHAR(10) NOT NULL,
    trait_name VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Player sessions - each game session gets a unique ID
CREATE TABLE IF NOT EXISTS player_sessions (
    id VARCHAR(36) PRIMARY KEY,
    started_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP NULL
);

-- Player responses - tracks each answer
CREATE TABLE IF NOT EXISTS responses (
    id INT AUTO_INCREMENT PRIMARY KEY,
    session_id VARCHAR(36) NOT NULL,
    question_id INT NOT NULL,
    choice ENUM('A', 'B') NOT NULL,
    answered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES player_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (question_id) REFERENCES questions(id) ON DELETE CASCADE
);

-- Personality scores - cumulative traits per session
CREATE TABLE IF NOT EXISTS personality_scores (
    id INT AUTO_INCREMENT PRIMARY KEY,
    session_id VARCHAR(36) NOT NULL,
    trait_name VARCHAR(50) NOT NULL,
    trait_value VARCHAR(50) NOT NULL,
    score INT DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES player_sessions(id) ON DELETE CASCADE,
    UNIQUE KEY unique_session_trait (session_id, trait_name)
);

-- Seed the first question
INSERT INTO questions (question_text, option_a_text, option_a_emoji, option_b_text, option_b_emoji, trait_name) 
VALUES (
    'There are 2 types of people in the world...',
    'Early Bird',
    '🌅',
    'Night Owl', 
    '🦉',
    'sleep_schedule'
);

-- Add more questions for future expansion
INSERT INTO questions (question_text, option_a_text, option_a_emoji, option_b_text, option_b_emoji, trait_name) 
VALUES 
    ('When it comes to planning...', 'Planner', '📋', 'Spontaneous', '🎲', 'planning_style'),
    ('At a party, you are...', 'The Host', '🎉', 'The Guest', '🍷', 'social_role'),
    ('Your desk is...', 'Organized', '📁', 'Creative Chaos', '🌪️', 'organization'),
    ('You prefer...', 'Sweet', '🍰', 'Savory', '🧀', 'taste_preference');
