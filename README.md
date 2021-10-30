# Java Classfile Parser

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)

A parser for [Java Classfiles](https://docs.oracle.com/javase/specs/jvms/se10/html/jvms-4.html), written in Rust using [nom](https://github.com/Geal/nom).

## Installation

Classfile Parser is available from crates.io and can be included in your Cargo enabled project like this:
 

## Implementation Status

- [x] Header
  - [x] Magic const
  - [x] Version info
- [x] Constant pool
  - [x] Constant pool size
  - [x] Constant types
    - [x] Utf8
    - [x] Integer
    - [x] Float
    - [x] Long
    - [x] Double
    - [x] Class
    - [x] String
    - [x] Fieldref
    - [x] Methodref
    - [x] InterfaceMethodref
    - [x] NameAndType
    - [x] MethodHandle
    - [x] MethodType
    - [x] InvokeDynamic
- [x] Access flags
- [x] This class
- [x] Super class
- [x] Interfaces
- [x] Fields
- [x] Methods
- [x] Attributes
  - [x] Basic attribute info block parsing
  - [ ] Known typed attributes parsing
    - [x] Critical for JVM
      - [x] ConstantValue
      - [x] Code
      - [x] StackMapTable
      - [x] Exceptions
      - [x] BootstrapMethods
    - [ ] Critical for Java SE
      - [ ] InnerClasses
      - [ ] EnclosingMethod
      - [ ] Synthetic
      - [ ] Signature
      - [ ] RuntimeVisibleAnnotations
      - [ ] RuntimeInvisibleAnnotations
      - [ ] RuntimeVisibleParameterAnnotations
      - [ ] RuntimeInvisibleParameterAnnotations
      - [ ] RuntimeVisibleTypeAnnotations
      - [ ] RuntimeInvisibleTypeAnnotations
      - [ ] AnnotationDefault
      - [ ] MethodParameters
    - [ ] Useful but not critical
      - [x] SourceFile
      - [ ] SourceDebugExtension
      - [ ] LineNumberTable
      - [ ] LocalVariableTable
      - [ ] LocalVariableTypeTable
      - [ ] Deprecated
